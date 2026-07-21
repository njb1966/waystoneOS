#include "cli_adapter.h"

#include <QByteArray>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonParseError>
#include <QProcess>
#include <QSet>

#include <algorithm>

namespace {

QJsonObject parseJsonObject(const QByteArray &data, QString *error) {
    QJsonParseError parseError;
    const QJsonDocument document = QJsonDocument::fromJson(data, &parseError);
    if (parseError.error != QJsonParseError::NoError || !document.isObject()) {
        if (error != nullptr) {
            *error = "could not parse command JSON";
        }
        return {};
    }
    return document.object();
}

QStringList jsonStringArray(const QJsonArray &array) {
    QStringList values;
    for (const auto &item : array) {
        values.append(item.toString());
    }
    return values;
}

QString renderValidationIssues(const QString &label, const QJsonArray &issues) {
    if (issues.isEmpty()) {
        return "  none\n";
    }

    QString text;
    for (const auto &item : issues) {
        const QJsonObject issue = item.toObject();
        const QString code = issue.value("code").toString("unknown");
        const QString message = issue.value("message").toString();
        text += QString("  %1: %2\n").arg(code, message);
    }
    return text.isEmpty() ? "  no " + label + "\n" : text;
}

QList<FeedEntryDiagnostic> feedDiagnostics(const QJsonArray &array) {
    QList<FeedEntryDiagnostic> diagnostics;
    for (const auto &item : array) {
        const QJsonObject object = item.toObject();
        FeedEntryDiagnostic diagnostic;
        diagnostic.path = object.value("path").toString();
        diagnostic.issues = jsonStringArray(object.value("issues").toArray());
        diagnostics.append(diagnostic);
    }
    return diagnostics;
}

QList<PublishValidationIssue> publishValidationIssues(const QJsonArray &array) {
    QList<PublishValidationIssue> issues;
    for (const auto &item : array) {
        const QJsonObject object = item.toObject();
        PublishValidationIssue issue;
        issue.code = object.value("code").toString();
        issue.message = object.value("message").toString();
        issue.path = object.value("path").toString();
        issues.append(issue);
    }
    return issues;
}

QString resolutionText(const QJsonObject &object) {
    if (object.isEmpty()) {
        return "none";
    }

    const QString id = object.value("id").toString();
    const QString status = object.value("status").toString();
    const QString detail = object.value("detail").toString();
    return QString("%1 (%2) - %3").arg(id, status, detail);
}

QStringList publishPlanArgs(const QString &mode, const QString &path,
                            const QString &target, const WorkspaceConfig &config,
                            const QString &remoteStatePath = {}) {
    QStringList args{mode,
                     "--project",
                     path,
                     "--target",
                     target,
                     "--hosts",
                     config.hostsRoot,
                     "--identities",
                     config.identitiesRoot};
    const QString trimmedRemoteState = remoteStatePath.trimmed();
    if (!trimmedRemoteState.isEmpty()) {
        args.append({"--remote-state", trimmedRemoteState});
    }
    args.append("--json");
    return args;
}

QStringList publishPlanArgs(const QString &mode, const QString &path,
                            const QString &target, const QString &date,
                            const WorkspaceConfig &config,
                            const QString &remoteStatePath = {}) {
    QStringList args = publishPlanArgs(mode, path, target, config, remoteStatePath);
    const int jsonIndex = args.indexOf("--json");
    args.insert(jsonIndex, "--date");
    args.insert(jsonIndex + 1, date);
    return args;
}

bool rootMissing(const QString &path, const QString &label, QString *error) {
    if (QFileInfo::exists(path)) {
        return false;
    }

    if (error != nullptr) {
        *error = "Configured " + label + " root not found: " + path;
    }
    return true;
}

QString absoluteWorkspacePath(const WorkspaceConfig &config, const QString &path) {
    const QFileInfo info(path);
    if (info.isAbsolute()) {
        return QDir::cleanPath(path);
    }
    return QDir(config.repoRoot).absoluteFilePath(path);
}

} // namespace

CliAdapter::CliAdapter(WorkspaceConfig config) : config_(std::move(config)) {}

QList<ProjectSummary> CliAdapter::listProjects(QString *error) const {
    if (rootMissing(config_.projectsRoot, "projects", error)) {
        return {};
    }

    const CommandResult result =
        runCommand("project", {"list", "--json", config_.projectsRoot});

    if (!result.error.isEmpty()) {
        if (error != nullptr) {
            *error = result.error;
        }
        return {};
    }

    if (result.exitCode != 0) {
        if (error != nullptr) {
            *error = commandFailureDetail(result, "project list failed");
        }
        return {};
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        if (error != nullptr) {
            *error = parseError;
        }
        return {};
    }

    QList<ProjectSummary> projects;
    const QJsonArray items = root.value("data").toObject().value("projects").toArray();
    for (const auto &item : items) {
        const QJsonObject object = item.toObject();
        ProjectSummary project;
        project.id = object.value("id").toString();
        project.name = object.value("name").toString();
        project.type = object.value("type").toString();
        project.path = object.value("path").toString();
        project.validation = projectValidationState(project.path);
        projects.append(project);
    }

    return projects;
}

ProjectCreateResult CliAdapter::createProject(const QString &id, const QString &name,
                                              const QString &projectType) const {
    ProjectCreateResult created;
    QString error;
    if (rootMissing(config_.projectsRoot, "projects", &error)) {
        created.error = error;
        return created;
    }

    const CommandResult result = runCommand(
        "project", {"create", "--json", config_.projectsRoot, id, name, projectType});

    if (!result.error.isEmpty()) {
        created.error = result.error;
        return created;
    }

    if (result.exitCode != 0) {
        created.error = commandFailureDetail(result, "project create failed");
        return created;
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        created.error = "project create returned unreadable JSON";
        return created;
    }

    created.projectPath =
        root.value("data").toObject().value("project_path").toString();
    if (created.projectPath.isEmpty()) {
        created.error = "project create did not return a project path";
        return created;
    }

    created.ok = true;
    return created;
}

ProjectTargetResult CliAdapter::addRemovablePublishTarget(
    const QString &projectPath, const QString &name, const QString &exportPath) const {
    ProjectTargetResult target;
    const CommandResult result =
        runCommand("project", {"target", "add-removable", "--json", projectPath, name, exportPath});

    if (!result.error.isEmpty()) {
        target.error = result.error;
        return target;
    }

    if (result.exitCode != 0) {
        target.error = commandFailureDetail(result, "project target add-removable failed");
        return target;
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        target.error = "project target add-removable returned unreadable JSON";
        return target;
    }

    const QJsonObject data = root.value("data").toObject();
    target.name = data.value("name").toString();
    target.method = data.value("method").toString();
    target.path = data.value("path").toString();
    target.ok = true;
    return target;
}

QString CliAdapter::inspectProject(const QString &path) const {
    const CommandResult result = runCommand("project", {"inspect", "--json", path});
    if (result.exitCode != 0) {
        return "Project inspection failed\n" +
               commandFailureDetail(result, "project inspect failed");
    }

    QString error;
    const QJsonObject root = parseJsonObject(result.standardOutput, &error);
    if (!error.isEmpty()) {
        return "Project inspection returned unreadable JSON";
    }

    const QJsonObject data = root.value("data").toObject();
    const QStringList targets = jsonStringArray(data.value("publish_targets").toArray());
    const QString targetText = targets.isEmpty() ? "none" : targets.join(", ");
    return QString("Project: %1\nID: %2\nType: %3\nSchema: %4\nContent: %5/%6\nPublish targets: %7")
        .arg(data.value("name").toString())
        .arg(data.value("id").toString())
        .arg(data.value("type").toString())
        .arg(data.value("project_schema").toInt())
        .arg(data.value("content_root").toString())
        .arg(data.value("content_index").toString())
        .arg(targetText);
}

QStringList CliAdapter::projectPublishTargets(const QString &path, QString *error) const {
    const CommandResult result = runCommand("project", {"inspect", "--json", path});
    if (result.exitCode != 0) {
        if (error != nullptr) {
            *error = commandFailureDetail(result, "project inspect failed");
        }
        return {};
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        if (error != nullptr) {
            *error = "project inspect returned unreadable JSON";
        }
        return {};
    }

    return jsonStringArray(root.value("data").toObject().value("publish_targets").toArray());
}

QString CliAdapter::projectValidationState(const QString &path) const {
    const CommandResult result = runCommand("project", {"validate", "--json", path});
    if (result.exitCode != 0) {
        return "invalid";
    }

    QString error;
    const QJsonObject root = parseJsonObject(result.standardOutput, &error);
    if (!error.isEmpty()) {
        return "unknown";
    }

    const QJsonObject data = root.value("data").toObject();
    return data.value("valid").toBool(false) ? "valid" : "invalid";
}

ProjectDocument CliAdapter::loadProjectDocument(const QString &path) const {
    ProjectDocument document;
    document.projectPath = absoluteWorkspacePath(config_, path);

    const CommandResult result =
        runCommand("project", {"inspect", "--json", document.projectPath});
    if (result.exitCode != 0) {
        document.error = "Project inspection failed\n" +
                         commandFailureDetail(result, "project inspect failed");
        return document;
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        document.error = "Project inspection returned unreadable JSON";
        return document;
    }

    const QJsonObject data = root.value("data").toObject();
    const QString contentRoot = data.value("content_root").toString();
    const QString contentIndex = data.value("content_index").toString();
    document.title = data.value("name").toString();
    document.contentRootPath =
        QDir(document.projectPath).absoluteFilePath(contentRoot);
    document.contentPath = QDir(document.contentRootPath).filePath(contentIndex);

    QFile file(document.contentPath);
    if (!file.open(QIODevice::ReadOnly | QIODevice::Text)) {
        document.error = "Content file could not be opened: " + document.contentPath;
        return document;
    }

    document.text = QString::fromUtf8(file.readAll());
    document.ok = true;
    return document;
}

bool CliAdapter::saveProjectDocument(const ProjectDocument &document,
                                     const QString &text, QString *error) const {
    if (!document.ok || document.contentPath.isEmpty()) {
        if (error != nullptr) {
            *error = "No project content file is loaded";
        }
        return false;
    }

    QFile file(document.contentPath);
    if (!file.open(QIODevice::WriteOnly | QIODevice::Text | QIODevice::Truncate)) {
        if (error != nullptr) {
            *error = "Content file could not be written: " + document.contentPath;
        }
        return false;
    }

    const QByteArray bytes = text.toUtf8();
    if (file.write(bytes) != bytes.size()) {
        if (error != nullptr) {
            *error = "Content write failed: " + document.contentPath;
        }
        return false;
    }

    return true;
}

PublishPreview CliAdapter::previewPublication(const QString &path, const QString &target,
                                              const QString &remoteStatePath) const {
    const CommandResult result =
        runCommand("publish", publishPlanArgs("--dry-run", path, target, config_,
                                              remoteStatePath));

    PublishPreview preview;
    if (!result.error.isEmpty()) {
        preview.error = result.error;
        return preview;
    }

    if (result.exitCode != 0) {
        preview.error = commandFailureDetail(result, "publish dry-run failed");
        return preview;
    }

    QString error;
    const QJsonObject root = parseJsonObject(result.standardOutput, &error);
    if (!error.isEmpty()) {
        preview.error = "publish dry-run returned unreadable JSON";
        return preview;
    }

    const QJsonObject data = root.value("data").toObject();
    preview.ok = true;
    preview.project = data.value("project").toString();
    preview.target = data.value("target").toString();
    preview.method = data.value("method").toString();
    preview.destination = data.value("destination").toString();
    preview.blocked = data.value("blocked").toBool(false);
    preview.hostResolution = resolutionText(data.value("host_resolution").toObject());
    preview.identityResolution =
        resolutionText(data.value("identity_resolution").toObject());
    const QJsonObject feed = data.value("feed").toObject();
    preview.feedConfigured = feed.value("configured").toBool(false);
    preview.feedEnabled = feed.value("enabled").toBool(false);
    preview.feedPath = feed.value("path").toString();
    preview.feedType = feed.value("type").toString();
    preview.feedExists = feed.value("exists").toBool(false);
    preview.feedPreparedEntries = feed.value("prepared_entries").toInt();
    preview.feedInvalidEntries = feed.value("invalid_entries").toInt();
    preview.feedDiagnostics =
        feedDiagnostics(feed.value("invalid_entry_diagnostics").toArray());
    const QJsonObject comparison = data.value("comparison").toObject();
    preview.comparisonConfigured = comparison.value("configured").toBool(false);
    preview.comparisonSource = comparison.value("source").toString();
    preview.comparisonRemotePaths = comparison.value("remote_paths").toInt();

    const QJsonObject changes = data.value("changes").toObject();
    preview.uploads = jsonStringArray(changes.value("upload").toArray());
    preview.updates = jsonStringArray(changes.value("update").toArray());
    preview.deletes = jsonStringArray(changes.value("delete").toArray());
    preview.skips = jsonStringArray(changes.value("skip").toArray());
    preview.verificationChecks =
        jsonStringArray(data.value("verification").toObject().value("checks").toArray());
    preview.confirmations = jsonStringArray(data.value("confirmations").toArray());
    return preview;
}

PublishValidationReport CliAdapter::validatePublication(
    const QString &path, const QString &target, const QString &remoteStatePath) const {
    const CommandResult result =
        runCommand("publish", publishPlanArgs("--validate", path, target, config_,
                                              remoteStatePath));

    PublishValidationReport report;
    if (!result.error.isEmpty()) {
        report.error = result.error;
        return report;
    }

    if (result.exitCode != 0) {
        report.error = commandFailureDetail(result, "publish validation failed");
        return report;
    }

    QString error;
    const QJsonObject root = parseJsonObject(result.standardOutput, &error);
    if (!error.isEmpty()) {
        report.error = "publish validation returned unreadable JSON";
        return report;
    }

    const QJsonObject data = root.value("data").toObject();
    report.ok = true;
    report.project = data.value("project").toString();
    report.target = data.value("target").toString();
    report.valid = data.value("valid").toBool(false);
    report.blocked = data.value("blocked").toBool(false);
    report.errors = publishValidationIssues(data.value("errors").toArray());
    report.warnings = publishValidationIssues(data.value("warnings").toArray());
    return report;
}

PlannedHistoryPreview CliAdapter::plannedPublicationHistory(const QString &path,
                                                            const QString &target,
                                                            const QString &date,
                                                            const QString &remoteStatePath) const {
    const CommandResult result =
        runCommand("publish", publishPlanArgs("--planned-history", path, target, date,
                                              config_, remoteStatePath));

    PlannedHistoryPreview preview;
    if (!result.error.isEmpty()) {
        preview.error = result.error;
        return preview;
    }

    if (result.exitCode != 0) {
        preview.error = commandFailureDetail(result, "publish planned-history failed");
        return preview;
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        preview.error = "publish planned-history returned unreadable JSON";
        return preview;
    }

    const QJsonObject data = root.value("data").toObject();
    preview.ok = true;
    preview.project = data.value("project").toString();
    preview.target = data.value("target").toString();
    preview.transferResult = data.value("transfer_result").toString();
    preview.verificationResult = data.value("verification_result").toString();
    preview.recordToml = data.value("record_toml").toString();

    const QJsonArray files = data.value("files").toArray();
    for (const auto &item : files) {
        const QJsonObject object = item.toObject();
        PlannedHistoryFile file;
        file.path = object.value("path").toString();
        file.action = object.value("action").toString();
        preview.files.append(file);
    }

    return preview;
}

PlannedHistorySaveResult CliAdapter::savePlannedPublicationHistoryPreview(
    const QString &path, const QString &target, const QString &date,
    const QString &remoteStatePath) const {
    const CommandResult result =
        runCommand("publish", publishPlanArgs("--save-planned-history-preview", path,
                                              target, date, config_, remoteStatePath));

    PlannedHistorySaveResult saved;
    if (!result.error.isEmpty()) {
        saved.error = result.error;
        return saved;
    }

    if (result.exitCode != 0) {
        saved.error =
            commandFailureDetail(result, "publish save planned-history preview failed");
        return saved;
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        saved.error = "publish save planned-history preview returned unreadable JSON";
        return saved;
    }

    const QJsonObject data = root.value("data").toObject();
    saved.ok = true;
    saved.project = data.value("project").toString();
    saved.target = data.value("target").toString();
    saved.outputPath = data.value("output_path").toString();
    return saved;
}

PlannedHistorySavedPreviewList
CliAdapter::listPlannedPublicationHistoryPreviews(const QString &path) const {
    const CommandResult result =
        runCommand("publish",
                   {"--list-planned-history-previews", "--project", path, "--json"});

    PlannedHistorySavedPreviewList list;
    if (!result.error.isEmpty()) {
        list.error = result.error;
        return list;
    }

    if (result.exitCode != 0) {
        list.error =
            commandFailureDetail(result, "publish list planned-history previews failed");
        return list;
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        list.error = "publish list planned-history previews returned unreadable JSON";
        return list;
    }

    const QJsonObject data = root.value("data").toObject();
    list.ok = true;
    list.projectPath = data.value("project_path").toString();

    const QJsonArray previews = data.value("previews").toArray();
    for (const auto &item : previews) {
        const QJsonObject object = item.toObject();
        PlannedHistorySavedPreview preview;
        preview.path = object.value("path").toString();
        preview.filename = object.value("filename").toString();
        preview.modifiedUnix = object.value("modified_unix").toInteger();
        preview.sizeBytes = object.value("size_bytes").toInteger();
        list.previews.append(preview);
    }

    return list;
}

PlannedHistorySavedPreviewDetail
CliAdapter::readPlannedPublicationHistoryPreview(const QString &projectPath,
                                                 const QString &previewPath) const {
    const CommandResult result =
        runCommand("publish",
                   {"--read-planned-history-preview",
                    "--project",
                    projectPath,
                    "--preview",
                    previewPath,
                    "--json"});

    PlannedHistorySavedPreviewDetail detail;
    if (!result.error.isEmpty()) {
        detail.error = result.error;
        return detail;
    }

    if (result.exitCode != 0) {
        detail.error =
            commandFailureDetail(result, "publish read planned-history preview failed");
        return detail;
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        detail.error = "publish read planned-history preview returned unreadable JSON";
        return detail;
    }

    const QJsonObject data = root.value("data").toObject();
    detail.ok = true;
    detail.projectPath = data.value("project_path").toString();
    detail.path = data.value("path").toString();
    detail.filename = data.value("filename").toString();
    detail.modifiedUnix = data.value("modified_unix").toInteger();
    detail.sizeBytes = data.value("size_bytes").toInteger();
    detail.recordToml = data.value("record_toml").toString();
    return detail;
}

CompletedHistorySaveResult CliAdapter::saveCompletedPublicationHistory(
    const QString &path, const QString &target, const QString &date,
    const QString &transferResult, const QString &verificationResult,
    bool rollbackAvailable, const QString &rollbackNotes) const {
    const CommandResult result = runCommand(
        "publish",
        {"--save-completed-history",
         "--project",
         path,
         "--target",
         target,
         "--date",
         date,
         "--transfer-result",
         transferResult,
         "--verification-result",
         verificationResult,
         "--rollback-available",
         rollbackAvailable ? "true" : "false",
         "--rollback-notes",
         rollbackNotes,
         "--hosts",
         config_.hostsRoot,
         "--identities",
         config_.identitiesRoot,
         "--json"});

    CompletedHistorySaveResult saved;
    if (!result.error.isEmpty()) {
        saved.error = result.error;
        return saved;
    }

    if (result.exitCode != 0) {
        saved.error = commandFailureDetail(result, "publish save completed-history failed");
        return saved;
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        saved.error = "publish save completed-history returned unreadable JSON";
        return saved;
    }

    const QJsonObject data = root.value("data").toObject();
    saved.ok = true;
    saved.project = data.value("project").toString();
    saved.target = data.value("target").toString();
    saved.outputPath = data.value("output_path").toString();
    return saved;
}

CompletedHistoryRecordList
CliAdapter::listCompletedPublicationHistory(const QString &path) const {
    const CommandResult result =
        runCommand("publish", {"--list-completed-history", "--project", path, "--json"});

    CompletedHistoryRecordList list;
    if (!result.error.isEmpty()) {
        list.error = result.error;
        return list;
    }

    if (result.exitCode != 0) {
        list.error = commandFailureDetail(result, "publish list completed-history failed");
        return list;
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        list.error = "publish list completed-history returned unreadable JSON";
        return list;
    }

    const QJsonObject data = root.value("data").toObject();
    list.ok = true;
    list.projectPath = data.value("project_path").toString();

    const QJsonArray records = data.value("records").toArray();
    for (const auto &item : records) {
        const QJsonObject object = item.toObject();
        CompletedHistoryRecord record;
        record.path = object.value("path").toString();
        record.filename = object.value("filename").toString();
        record.modifiedUnix = object.value("modified_unix").toInteger();
        record.sizeBytes = object.value("size_bytes").toInteger();
        list.records.append(record);
    }

    return list;
}

CompletedHistoryRecordDetail CliAdapter::readCompletedPublicationHistory(
    const QString &projectPath, const QString &recordPath) const {
    const CommandResult result =
        runCommand("publish",
                   {"--read-completed-history",
                    "--project",
                    projectPath,
                    "--record",
                    recordPath,
                    "--json"});

    CompletedHistoryRecordDetail detail;
    if (!result.error.isEmpty()) {
        detail.error = result.error;
        return detail;
    }

    if (result.exitCode != 0) {
        detail.error = commandFailureDetail(result, "publish read completed-history failed");
        return detail;
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        detail.error = "publish read completed-history returned unreadable JSON";
        return detail;
    }

    const QJsonObject data = root.value("data").toObject();
    detail.ok = true;
    detail.projectPath = data.value("project_path").toString();
    detail.path = data.value("path").toString();
    detail.filename = data.value("filename").toString();
    detail.modifiedUnix = data.value("modified_unix").toInteger();
    detail.sizeBytes = data.value("size_bytes").toInteger();
    detail.recordToml = data.value("record_toml").toString();
    return detail;
}

QList<HostSummary> CliAdapter::listHosts(QString *error) const {
    if (rootMissing(config_.hostsRoot, "hosts", error)) {
        return {};
    }

    const CommandResult result = runCommand(
        "host", {"list", "--json", config_.hostsRoot});

    if (!result.error.isEmpty()) {
        if (error != nullptr) {
            *error = result.error;
        }
        return {};
    }

    if (result.exitCode != 0) {
        if (error != nullptr) {
            *error = commandFailureDetail(result, "host list failed");
        }
        return {};
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        if (error != nullptr) {
            *error = parseError;
        }
        return {};
    }

    QList<HostSummary> hosts;
    const QJsonArray items = root.value("data").toObject().value("hosts").toArray();
    for (const auto &item : items) {
        const QJsonObject object = item.toObject();
        HostSummary host;
        host.id = object.value("id").toString();
        host.displayName = object.value("display_name").toString();
        host.address = object.value("address").toString();
        host.path = object.value("path").toString();
        host.validation = hostValidationState(host.path);
        hosts.append(host);
    }

    return hosts;
}

QString CliAdapter::inspectHost(const QString &path) const {
    const CommandResult result = runCommand("host", {"inspect", "--json", path});
    if (result.exitCode != 0) {
        return "Host inspection failed\n" +
               commandFailureDetail(result, "host inspect failed");
    }

    QString error;
    const QJsonObject root = parseJsonObject(result.standardOutput, &error);
    if (!error.isEmpty()) {
        return "Host inspection returned unreadable JSON";
    }

    const QJsonObject data = root.value("data").toObject();
    return QString("Host: %1\nID: %2\nAddress: %3\nServices: %4")
        .arg(data.value("display_name").toString())
        .arg(data.value("id").toString())
        .arg(data.value("address").toString())
        .arg(data.value("services").toInt());
}

QString CliAdapter::hostValidationState(const QString &path) const {
    const CommandResult result = runCommand("host", {"validate", "--json", path});
    if (result.exitCode != 0) {
        return "invalid";
    }

    QString error;
    const QJsonObject root = parseJsonObject(result.standardOutput, &error);
    if (!error.isEmpty()) {
        return "unknown";
    }

    const QJsonObject data = root.value("data").toObject();
    return data.value("valid").toBool(false) ? "valid" : "invalid";
}

QList<IdentitySummary> CliAdapter::listIdentities(QString *error) const {
    if (rootMissing(config_.identitiesRoot, "identities", error)) {
        return {};
    }

    const CommandResult result =
        runCommand("identity", {"list", "--json", config_.identitiesRoot});

    if (!result.error.isEmpty()) {
        if (error != nullptr) {
            *error = result.error;
        }
        return {};
    }

    if (result.exitCode != 0) {
        if (error != nullptr) {
            *error = commandFailureDetail(result, "identity list failed");
        }
        return {};
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        if (error != nullptr) {
            *error = parseError;
        }
        return {};
    }

    QList<IdentitySummary> identities;
    const QJsonArray items = root.value("data").toObject().value("identities").toArray();
    for (const auto &item : items) {
        const QJsonObject object = item.toObject();
        IdentitySummary identity;
        identity.id = object.value("id").toString();
        identity.displayName = object.value("display_name").toString();
        identity.path = object.value("path").toString();
        identity.validation = identityValidationState(identity.path);
        identities.append(identity);
    }

    return identities;
}

QString CliAdapter::inspectIdentity(const QString &path) const {
    const CommandResult result = runCommand("identity", {"inspect", "--json", path});
    if (result.exitCode != 0) {
        return "Identity inspection failed\n" +
               commandFailureDetail(result, "identity inspect failed");
    }

    QString error;
    const QJsonObject root = parseJsonObject(result.standardOutput, &error);
    if (!error.isEmpty()) {
        return "Identity inspection returned unreadable JSON";
    }

    const QJsonObject data = root.value("data").toObject();
    return QString("Identity: %1\nID: %2\nSSH keys: %3\nCertificates: %4")
        .arg(data.value("display_name").toString())
        .arg(data.value("id").toString())
        .arg(data.value("ssh_keys").toInt())
        .arg(data.value("certificates").toInt());
}

QString CliAdapter::identityValidationState(const QString &path) const {
    const CommandResult result = runCommand("identity", {"validate", "--json", path});
    if (result.exitCode != 0) {
        return "invalid";
    }

    QString error;
    const QJsonObject root = parseJsonObject(result.standardOutput, &error);
    if (!error.isEmpty()) {
        return "unknown";
    }

    const QJsonObject data = root.value("data").toObject();
    return data.value("valid").toBool(false) ? "valid" : "invalid";
}

QList<RecordingSummary> CliAdapter::listRecordings(QString *error) const {
    if (rootMissing(config_.audioMetadataRoot, "audio metadata", error)) {
        return {};
    }

    const CommandResult result =
        runCommand("record", {"list", "--json", config_.audioMetadataRoot});

    if (!result.error.isEmpty()) {
        if (error != nullptr) {
            *error = result.error;
        }
        return {};
    }

    if (result.exitCode != 0) {
        if (error != nullptr) {
            *error = commandFailureDetail(result, "record list failed");
        }
        return {};
    }

    QSet<QString> playableIds;
    const CommandResult playableResult =
        runCommand("listen", {"library", "--json", config_.audioMetadataRoot});
    if (playableResult.exitCode == 0) {
        QString playableParseError;
        const QJsonObject playableRoot =
            parseJsonObject(playableResult.standardOutput, &playableParseError);
        const QJsonArray playableItems =
            playableRoot.value("data").toObject().value("recordings").toArray();
        for (const auto &item : playableItems) {
            playableIds.insert(item.toObject().value("id").toString());
        }
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        if (error != nullptr) {
            *error = parseError;
        }
        return {};
    }

    QList<RecordingSummary> recordings;
    const QJsonArray items = root.value("data").toObject().value("recordings").toArray();
    for (const auto &item : items) {
        const QJsonObject object = item.toObject();
        RecordingSummary recording;
        recording.id = object.value("id").toString();
        recording.title = object.value("title").toString();
        recording.path = object.value("path").toString();
        recording.validation = recordingValidationState(recording.path);
        recording.playable = playableIds.contains(recording.id) ? "yes" : "no";
        recordings.append(recording);
    }

    return recordings;
}

RecordingCaptureResult CliAdapter::captureRecording(
    const QString &projectPath, const QString &master, int durationSeconds,
    const QString &inputFormat, const QString &input) const {
    RecordingCaptureResult captured;
    const int timeoutMs = std::max(5000, (durationSeconds + 5) * 1000);
    const CommandResult result =
        runCommand("record", {"capture", "--json", projectPath, master,
                              QString::number(durationSeconds), inputFormat, input},
                   timeoutMs);

    if (!result.error.isEmpty()) {
        captured.error = result.error;
        return captured;
    }

    if (result.exitCode != 0) {
        captured.error = commandFailureDetail(result, "record capture failed");
        return captured;
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        captured.error = "record capture returned unreadable JSON";
        return captured;
    }

    const QJsonObject data = root.value("data").toObject();
    captured.master = data.value("master").toString();
    captured.outputPath = data.value("output_path").toString();
    captured.outputRelativePath = data.value("output_relative_path").toString();
    captured.durationSeconds = data.value("duration_seconds").toInt();
    captured.channels = data.value("channels").toInt();
    captured.sampleRate = data.value("sample_rate").toInt();
    captured.format = data.value("format").toString();
    captured.engine = data.value("engine").toString();
    captured.ok = true;
    return captured;
}

RecordingExportResult CliAdapter::exportOpusPublicationCopy(
    const QString &projectPath, const QString &master, const QString &published,
    const QString &preset) const {
    RecordingExportResult exported;
    const CommandResult result =
        runCommand("record", {"export-opus", "--json", projectPath, master,
                              published, preset});

    if (!result.error.isEmpty()) {
        exported.error = result.error;
        return exported;
    }

    if (result.exitCode != 0) {
        exported.error =
            commandFailureDetail(result, "record export-opus failed");
        return exported;
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        exported.error = "record export-opus returned unreadable JSON";
        return exported;
    }

    const QJsonObject data = root.value("data").toObject();
    exported.master = data.value("master").toString();
    exported.published = data.value("published").toString();
    exported.outputPath = data.value("output_path").toString();
    exported.outputRelativePath = data.value("output_relative_path").toString();
    exported.preset = data.value("preset").toString();
    exported.mimeType = data.value("mime_type").toString();
    exported.engine = data.value("engine").toString();
    exported.ok = true;
    return exported;
}

RecordingAttachResult CliAdapter::attachRecording(
    const QString &projectPath, const QString &id, const QString &title,
    const QString &master, const QString &published, const QString &feed,
    const QString &entryId, const QString &mimeType) const {
    RecordingAttachResult attached;
    const CommandResult result =
        runCommand("record", {"attach", "--json", projectPath, id, title, master,
                              published, feed, entryId, mimeType});

    if (!result.error.isEmpty()) {
        attached.error = result.error;
        return attached;
    }

    if (result.exitCode != 0) {
        attached.error = commandFailureDetail(result, "record attach failed");
        return attached;
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        attached.error = "record attach returned unreadable JSON";
        return attached;
    }

    const QJsonObject data = root.value("data").toObject();
    attached.id = data.value("id").toString();
    attached.title = data.value("title").toString();
    attached.metadataPath = data.value("metadata_path").toString();
    attached.metadataRelativePath =
        data.value("metadata_relative_path").toString();
    attached.master = data.value("master").toString();
    attached.published = data.value("published").toString();
    attached.feed = data.value("feed").toString();
    attached.ok = true;
    return attached;
}

RecordingUpdateResult CliAdapter::updateRecording(
    const QString &projectPath, const QString &id, const QString &title,
    const QString &master, const QString &published, const QString &feed,
    const QString &entryId, const QString &mimeType) const {
    RecordingUpdateResult updated;
    const CommandResult result =
        runCommand("record", {"update", "--json", projectPath, id, title, master,
                              published, feed, entryId, mimeType});

    if (!result.error.isEmpty()) {
        updated.error = result.error;
        return updated;
    }

    if (result.exitCode != 0) {
        updated.error = commandFailureDetail(result, "record update failed");
        return updated;
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        updated.error = "record update returned unreadable JSON";
        return updated;
    }

    const QJsonObject data = root.value("data").toObject();
    updated.id = data.value("id").toString();
    updated.title = data.value("title").toString();
    updated.metadataPath = data.value("metadata_path").toString();
    updated.metadataRelativePath =
        data.value("metadata_relative_path").toString();
    updated.master = data.value("master").toString();
    updated.published = data.value("published").toString();
    updated.feed = data.value("feed").toString();
    updated.ok = true;
    return updated;
}

FeedEntryPrepareResult CliAdapter::prepareFeedEntry(
    const QString &projectPath, const QString &recordingId, const QString &updated,
    const QString &summary) const {
    FeedEntryPrepareResult prepared;
    const CommandResult result =
        runCommand("record", {"prepare-feed-entry", "--json", projectPath,
                              recordingId, updated, summary});

    if (!result.error.isEmpty()) {
        prepared.error = result.error;
        return prepared;
    }

    if (result.exitCode != 0) {
        prepared.error = commandFailureDetail(result, "record prepare-feed-entry failed");
        return prepared;
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        prepared.error = "record prepare-feed-entry returned unreadable JSON";
        return prepared;
    }

    const QJsonObject data = root.value("data").toObject();
    prepared.recordingId = data.value("recording_id").toString();
    prepared.title = data.value("title").toString();
    prepared.outputPath = data.value("output_path").toString();
    prepared.outputRelativePath = data.value("output_relative_path").toString();
    prepared.published = data.value("published").toString();
    prepared.feed = data.value("feed").toString();
    prepared.ok = true;
    return prepared;
}

FeedEntryPrepareResult CliAdapter::updateFeedEntry(
    const QString &projectPath, const QString &recordingId, const QString &updated,
    const QString &summary) const {
    FeedEntryPrepareResult prepared;
    const CommandResult result =
        runCommand("record", {"update-feed-entry", "--json", projectPath,
                              recordingId, updated, summary});

    if (!result.error.isEmpty()) {
        prepared.error = result.error;
        return prepared;
    }

    if (result.exitCode != 0) {
        prepared.error = commandFailureDetail(result, "record update-feed-entry failed");
        return prepared;
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        prepared.error = "record update-feed-entry returned unreadable JSON";
        return prepared;
    }

    const QJsonObject data = root.value("data").toObject();
    prepared.recordingId = data.value("recording_id").toString();
    prepared.title = data.value("title").toString();
    prepared.outputPath = data.value("output_path").toString();
    prepared.outputRelativePath = data.value("output_relative_path").toString();
    prepared.published = data.value("published").toString();
    prepared.feed = data.value("feed").toString();
    prepared.ok = true;
    return prepared;
}

FeedGenerateResult CliAdapter::generateFeed(const QString &projectPath) const {
    FeedGenerateResult generated;
    const CommandResult result =
        runCommand("record", {"generate-feed", "--json", projectPath});

    if (!result.error.isEmpty()) {
        generated.error = result.error;
        return generated;
    }

    if (result.exitCode != 0) {
        generated.error = commandFailureDetail(result, "record generate-feed failed");
        return generated;
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    if (!parseError.isEmpty()) {
        generated.error = "record generate-feed returned unreadable JSON";
        return generated;
    }

    const QJsonObject data = root.value("data").toObject();
    generated.feedPath = data.value("feed_path").toString();
    generated.feedRelativePath = data.value("feed_relative_path").toString();
    generated.entries = data.value("entries").toInt();
    generated.updated = data.value("updated").toString();
    generated.ok = true;
    return generated;
}

QString CliAdapter::inspectRecording(const QString &path) const {
    const CommandResult result = runCommand("record", {"inspect", "--json", path});
    if (result.exitCode != 0) {
        return "Recording inspection failed\n" +
               commandFailureDetail(result, "record inspect failed");
    }

    QString error;
    const QJsonObject root = parseJsonObject(result.standardOutput, &error);
    if (!error.isEmpty()) {
        return "Recording inspection returned unreadable JSON";
    }

    const QJsonObject data = root.value("data").toObject();
    return QString("Recording: %1\nID: %2\nMaster: %3\nPublished: %4")
        .arg(data.value("title").toString())
        .arg(data.value("id").toString())
        .arg(data.value("master").toString())
        .arg(data.value("published").toString("none"));
}

QString CliAdapter::publicationValidationState(const QString &projectPath,
                                               const QString &recordingId) const {
    const CommandResult result =
        runCommand("record", {"validate-publication", "--json", projectPath, recordingId});
    if (result.exitCode != 0) {
        return "invalid";
    }

    QString error;
    const QJsonObject root = parseJsonObject(result.standardOutput, &error);
    if (!error.isEmpty()) {
        return "unknown";
    }

    const QJsonObject data = root.value("data").toObject();
    return data.value("valid").toBool(false) ? "valid" : "invalid";
}

QString CliAdapter::feedEntryValidationState(const QString &projectPath,
                                             const QString &recordingId) const {
    const CommandResult result =
        runCommand("record", {"validate-feed-entry", "--json", projectPath, recordingId});
    if (result.exitCode != 0) {
        return "invalid";
    }

    QString error;
    const QJsonObject root = parseJsonObject(result.standardOutput, &error);
    if (!error.isEmpty()) {
        return "unknown";
    }

    const QJsonObject data = root.value("data").toObject();
    return data.value("valid").toBool(false) ? "valid" : "invalid";
}

QString CliAdapter::feedEntryValidationDetail(const QString &projectPath,
                                              const QString &feedEntryPath) const {
    const QString recordingId = QFileInfo(feedEntryPath).completeBaseName();
    if (recordingId.trimmed().isEmpty()) {
        return "Feed-entry validation failed\nCould not derive recording ID from " +
               feedEntryPath;
    }

    const CommandResult result =
        runCommand("record", {"validate-feed-entry", "--json", projectPath, recordingId});
    if (!result.error.isEmpty()) {
        return "Feed-entry validation failed\n" + result.error;
    }

    QString error;
    const QJsonObject root = parseJsonObject(result.standardOutput, &error);
    if (!error.isEmpty()) {
        if (result.exitCode != 0) {
            return "Feed-entry validation failed\n" +
                   commandFailureDetail(result, "record validate-feed-entry failed");
        }
        return "Feed-entry validation returned unreadable JSON";
    }

    const QJsonObject data = root.value("data").toObject();
    QString text;
    text += "Feed-entry validation\n";
    text += "Path: " + feedEntryPath + "\n";
    text += "Recording ID: " + recordingId + "\n";
    text += "Valid: " + QString(data.value("valid").toBool(false) ? "yes" : "no") + "\n";
    text += "Errors:\n";
    text += renderValidationIssues("errors", data.value("errors").toArray());
    text += "Warnings:\n";
    text += renderValidationIssues("warnings", data.value("warnings").toArray());
    return text.trimmed();
}

QString CliAdapter::recordingValidationState(const QString &path) const {
    const CommandResult result = runCommand("record", {"validate", "--json", path});
    if (result.exitCode != 0) {
        return "invalid";
    }

    QString error;
    const QJsonObject root = parseJsonObject(result.standardOutput, &error);
    if (!error.isEmpty()) {
        return "unknown";
    }

    const QJsonObject data = root.value("data").toObject();
    return data.value("valid").toBool(false) ? "valid" : "invalid";
}

QString CliAdapter::commandProgram(const QString &binaryName) const {
    const QString localBinary = QDir(config_.repoRoot).filePath("target/debug/" + binaryName);
    if (QFileInfo(localBinary).isExecutable()) {
        return localBinary;
    }
    return binaryName;
}

CommandResult CliAdapter::runCommand(const QString &binaryName,
                                     const QStringList &arguments,
                                     int timeoutMs) const {
    QProcess process;
    CommandResult result;
    process.setWorkingDirectory(config_.repoRoot);
    process.start(commandProgram(binaryName), arguments);

    if (!process.waitForStarted(3000)) {
        result.error = binaryName + " command did not start";
        return result;
    }

    if (!process.waitForFinished(timeoutMs)) {
        process.kill();
        process.waitForFinished(1000);
        result.error = binaryName + " command timed out";
        result.standardError = process.readAllStandardError();
        return result;
    }

    result.exitCode = process.exitCode();
    result.standardOutput = process.readAllStandardOutput();
    result.standardError = process.readAllStandardError();
    if (process.exitStatus() != QProcess::NormalExit) {
        result.error = binaryName + " command exited abnormally";
    }
    return result;
}

QString CliAdapter::commandFailureDetail(const CommandResult &result,
                                         const QString &fallback) const {
    const QString stderrText = QString::fromUtf8(result.standardError).trimmed();
    if (!stderrText.isEmpty()) {
        return stderrText;
    }

    QString parseError;
    const QJsonObject root = parseJsonObject(result.standardOutput, &parseError);
    const QJsonObject errorObject = root.value("error").toObject();
    const QString message = errorObject.value("message").toString();
    if (!message.isEmpty()) {
        return message;
    }

    if (!result.error.isEmpty()) {
        return result.error;
    }

    return fallback;
}
