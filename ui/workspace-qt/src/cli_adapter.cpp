#include "cli_adapter.h"

#include <QDir>
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

} // namespace

CliAdapter::CliAdapter(WorkspaceConfig config) : config_(std::move(config)) {}

QList<ProjectSummary> CliAdapter::listProjects(QString *error) const {
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
    return QString("Project: %1\nID: %2\nType: %3\nSchema: %4\nContent: %5/%6")
        .arg(data.value("name").toString())
        .arg(data.value("id").toString())
        .arg(data.value("type").toString())
        .arg(data.value("project_schema").toInt())
        .arg(data.value("content_root").toString())
        .arg(data.value("content_index").toString());
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

QList<HostSummary> CliAdapter::listHosts(QString *error) const {
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
