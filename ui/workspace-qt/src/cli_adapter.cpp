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

QString resolutionText(const QJsonObject &object) {
    if (object.isEmpty()) {
        return "none";
    }

    const QString id = object.value("id").toString();
    const QString status = object.value("status").toString();
    const QString detail = object.value("detail").toString();
    return QString("%1 (%2) - %3").arg(id, status, detail);
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

PublishPreview CliAdapter::previewPublication(const QString &path,
                                              const QString &target) const {
    const CommandResult result = runCommand(
        "publish",
        {"--dry-run",
         "--project",
         path,
         "--target",
         target,
         "--hosts",
         config_.hostsRoot,
         "--identities",
         config_.identitiesRoot,
         "--json"});

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

    const QJsonObject changes = data.value("changes").toObject();
    preview.uploads = jsonStringArray(changes.value("upload").toArray());
    preview.verificationChecks =
        jsonStringArray(data.value("verification").toObject().value("checks").toArray());
    preview.confirmations = jsonStringArray(data.value("confirmations").toArray());
    return preview;
}

PlannedHistoryPreview CliAdapter::plannedPublicationHistory(const QString &path,
                                                            const QString &target,
                                                            const QString &date) const {
    const CommandResult result = runCommand(
        "publish",
        {"--planned-history",
         "--project",
         path,
         "--target",
         target,
         "--date",
         date,
         "--hosts",
         config_.hostsRoot,
         "--identities",
         config_.identitiesRoot,
         "--json"});

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
    const QString &path, const QString &target, const QString &date) const {
    const CommandResult result = runCommand(
        "publish",
        {"--save-planned-history-preview",
         "--project",
         path,
         "--target",
         target,
         "--date",
         date,
         "--hosts",
         config_.hostsRoot,
         "--identities",
         config_.identitiesRoot,
         "--json"});

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
                                     const QStringList &arguments) const {
    QProcess process;
    CommandResult result;
    process.setWorkingDirectory(config_.repoRoot);
    process.start(commandProgram(binaryName), arguments);

    if (!process.waitForStarted(3000)) {
        result.error = binaryName + " command did not start";
        return result;
    }

    if (!process.waitForFinished(5000)) {
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
