#pragma once

#include <QByteArray>
#include <QList>
#include <QString>
#include <QStringList>

#include "workspace_config.h"

struct CommandResult {
    int exitCode = -1;
    QByteArray standardOutput;
    QByteArray standardError;
    QString error;
};

struct ProjectSummary {
    QString id;
    QString name;
    QString type;
    QString path;
    QString validation;
};

struct ProjectDocument {
    bool ok = false;
    QString projectPath;
    QString contentRootPath;
    QString contentPath;
    QString title;
    QString text;
    QString error;
};

struct ProjectCreateResult {
    bool ok = false;
    QString projectPath;
    QString error;
};

struct PublishPreview {
    bool ok = false;
    bool blocked = false;
    QString project;
    QString target;
    QString method;
    QString destination;
    QString hostResolution;
    QString identityResolution;
    QStringList uploads;
    QStringList verificationChecks;
    QStringList confirmations;
    QString error;
};

struct HostSummary {
    QString id;
    QString displayName;
    QString address;
    QString path;
    QString validation;
};

struct IdentitySummary {
    QString id;
    QString displayName;
    QString path;
    QString validation;
};

struct RecordingSummary {
    QString id;
    QString title;
    QString path;
    QString validation;
    QString playable;
};

class CliAdapter {
public:
    explicit CliAdapter(WorkspaceConfig config);

    QList<ProjectSummary> listProjects(QString *error) const;
    ProjectCreateResult createProject(const QString &id, const QString &name,
                                      const QString &projectType) const;
    QString inspectProject(const QString &path) const;
    QString projectValidationState(const QString &path) const;
    ProjectDocument loadProjectDocument(const QString &path) const;
    bool saveProjectDocument(const ProjectDocument &document, const QString &text,
                             QString *error) const;
    PublishPreview previewPublication(const QString &path, const QString &target) const;
    QList<HostSummary> listHosts(QString *error) const;
    QString inspectHost(const QString &path) const;
    QString hostValidationState(const QString &path) const;
    QList<IdentitySummary> listIdentities(QString *error) const;
    QString inspectIdentity(const QString &path) const;
    QString identityValidationState(const QString &path) const;
    QList<RecordingSummary> listRecordings(QString *error) const;
    QString inspectRecording(const QString &path) const;
    QString recordingValidationState(const QString &path) const;

private:
    WorkspaceConfig config_;

    QString commandProgram(const QString &binaryName) const;
    CommandResult runCommand(const QString &binaryName, const QStringList &arguments) const;
    QString commandFailureDetail(const CommandResult &result,
                                 const QString &fallback) const;
};
