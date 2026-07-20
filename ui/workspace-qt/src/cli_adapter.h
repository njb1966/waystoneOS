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

struct ProjectTargetResult {
    bool ok = false;
    QString name;
    QString method;
    QString path;
    QString error;
};

struct FeedEntryDiagnostic {
    QString path;
    QStringList issues;
};

struct PublishPreview {
    bool ok = false;
    bool blocked = false;
    bool feedConfigured = false;
    bool feedEnabled = false;
    bool feedExists = false;
    int feedPreparedEntries = 0;
    int feedInvalidEntries = 0;
    QString project;
    QString target;
    QString method;
    QString destination;
    QString feedPath;
    QString feedType;
    QList<FeedEntryDiagnostic> feedDiagnostics;
    QString hostResolution;
    QString identityResolution;
    QStringList uploads;
    QStringList verificationChecks;
    QStringList confirmations;
    QString error;
};

struct PublishValidationIssue {
    QString code;
    QString message;
    QString path;
};

struct PublishValidationReport {
    bool ok = false;
    bool valid = false;
    bool blocked = false;
    QString project;
    QString target;
    QList<PublishValidationIssue> errors;
    QList<PublishValidationIssue> warnings;
    QString error;
};

struct PlannedHistoryFile {
    QString path;
    QString action;
};

struct PlannedHistoryPreview {
    bool ok = false;
    QString project;
    QString target;
    QString transferResult;
    QString verificationResult;
    QList<PlannedHistoryFile> files;
    QString recordToml;
    QString error;
};

struct PlannedHistorySaveResult {
    bool ok = false;
    QString project;
    QString target;
    QString outputPath;
    QString error;
};

struct PlannedHistorySavedPreview {
    QString path;
    QString filename;
    qint64 modifiedUnix = 0;
    qint64 sizeBytes = 0;
};

struct PlannedHistorySavedPreviewList {
    bool ok = false;
    QString projectPath;
    QList<PlannedHistorySavedPreview> previews;
    QString error;
};

struct PlannedHistorySavedPreviewDetail {
    bool ok = false;
    QString projectPath;
    QString path;
    QString filename;
    qint64 modifiedUnix = 0;
    qint64 sizeBytes = 0;
    QString recordToml;
    QString error;
};

struct CompletedHistorySaveResult {
    bool ok = false;
    QString project;
    QString target;
    QString outputPath;
    QString error;
};

struct CompletedHistoryRecord {
    QString path;
    QString filename;
    qint64 modifiedUnix = 0;
    qint64 sizeBytes = 0;
};

struct CompletedHistoryRecordList {
    bool ok = false;
    QString projectPath;
    QList<CompletedHistoryRecord> records;
    QString error;
};

struct CompletedHistoryRecordDetail {
    bool ok = false;
    QString projectPath;
    QString path;
    QString filename;
    qint64 modifiedUnix = 0;
    qint64 sizeBytes = 0;
    QString recordToml;
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

struct RecordingExportResult {
    bool ok = false;
    QString master;
    QString published;
    QString outputPath;
    QString outputRelativePath;
    QString preset;
    QString mimeType;
    QString engine;
    QString error;
};

struct RecordingCaptureResult {
    bool ok = false;
    QString master;
    QString outputPath;
    QString outputRelativePath;
    int durationSeconds = 0;
    int channels = 0;
    int sampleRate = 0;
    QString format;
    QString engine;
    QString error;
};

struct RecordingAttachResult {
    bool ok = false;
    QString id;
    QString title;
    QString metadataPath;
    QString metadataRelativePath;
    QString master;
    QString published;
    QString feed;
    QString error;
};

struct RecordingUpdateResult {
    bool ok = false;
    QString id;
    QString title;
    QString metadataPath;
    QString metadataRelativePath;
    QString master;
    QString published;
    QString feed;
    QString error;
};

struct FeedEntryPrepareResult {
    bool ok = false;
    QString recordingId;
    QString title;
    QString outputPath;
    QString outputRelativePath;
    QString published;
    QString feed;
    QString error;
};

struct FeedGenerateResult {
    bool ok = false;
    QString feedPath;
    QString feedRelativePath;
    int entries = 0;
    QString updated;
    QString error;
};

class CliAdapter {
public:
    explicit CliAdapter(WorkspaceConfig config);

    QList<ProjectSummary> listProjects(QString *error) const;
    ProjectCreateResult createProject(const QString &id, const QString &name,
                                      const QString &projectType) const;
    ProjectTargetResult addRemovablePublishTarget(const QString &projectPath,
                                                  const QString &name,
                                                  const QString &exportPath) const;
    QString inspectProject(const QString &path) const;
    QStringList projectPublishTargets(const QString &path, QString *error) const;
    QString projectValidationState(const QString &path) const;
    ProjectDocument loadProjectDocument(const QString &path) const;
    bool saveProjectDocument(const ProjectDocument &document, const QString &text,
                             QString *error) const;
    PublishPreview previewPublication(const QString &path, const QString &target) const;
    PublishValidationReport validatePublication(const QString &path,
                                                const QString &target) const;
    PlannedHistoryPreview plannedPublicationHistory(const QString &path,
                                                    const QString &target,
                                                    const QString &date) const;
    PlannedHistorySaveResult savePlannedPublicationHistoryPreview(
        const QString &path, const QString &target, const QString &date) const;
    PlannedHistorySavedPreviewList listPlannedPublicationHistoryPreviews(
        const QString &path) const;
    PlannedHistorySavedPreviewDetail readPlannedPublicationHistoryPreview(
        const QString &projectPath, const QString &previewPath) const;
    CompletedHistorySaveResult saveCompletedPublicationHistory(
        const QString &path, const QString &target, const QString &date,
        const QString &transferResult, const QString &verificationResult,
        bool rollbackAvailable, const QString &rollbackNotes) const;
    CompletedHistoryRecordList listCompletedPublicationHistory(
        const QString &path) const;
    CompletedHistoryRecordDetail readCompletedPublicationHistory(
        const QString &projectPath, const QString &recordPath) const;
    QList<HostSummary> listHosts(QString *error) const;
    QString inspectHost(const QString &path) const;
    QString hostValidationState(const QString &path) const;
    QList<IdentitySummary> listIdentities(QString *error) const;
    QString inspectIdentity(const QString &path) const;
    QString identityValidationState(const QString &path) const;
    QList<RecordingSummary> listRecordings(QString *error) const;
    RecordingCaptureResult captureRecording(const QString &projectPath,
                                            const QString &master,
                                            int durationSeconds,
                                            const QString &inputFormat,
                                            const QString &input) const;
    RecordingExportResult exportOpusPublicationCopy(const QString &projectPath,
                                                    const QString &master,
                                                    const QString &published,
                                                    const QString &preset) const;
    RecordingAttachResult attachRecording(const QString &projectPath,
                                          const QString &id,
                                          const QString &title,
                                          const QString &master,
                                          const QString &published,
                                          const QString &feed,
                                          const QString &entryId,
                                          const QString &mimeType) const;
    RecordingUpdateResult updateRecording(const QString &projectPath,
                                          const QString &id,
                                          const QString &title,
                                          const QString &master,
                                          const QString &published,
                                          const QString &feed,
                                          const QString &entryId,
                                          const QString &mimeType) const;
    FeedEntryPrepareResult prepareFeedEntry(const QString &projectPath,
                                            const QString &recordingId,
                                            const QString &updated,
                                            const QString &summary) const;
    FeedEntryPrepareResult updateFeedEntry(const QString &projectPath,
                                           const QString &recordingId,
                                           const QString &updated,
                                           const QString &summary) const;
    FeedGenerateResult generateFeed(const QString &projectPath) const;
    QString publicationValidationState(const QString &projectPath,
                                       const QString &recordingId) const;
    QString feedEntryValidationState(const QString &projectPath,
                                     const QString &recordingId) const;
    QString feedEntryValidationDetail(const QString &projectPath,
                                      const QString &feedEntryPath) const;
    QString inspectRecording(const QString &path) const;
    QString recordingValidationState(const QString &path) const;

private:
    WorkspaceConfig config_;

    QString commandProgram(const QString &binaryName) const;
    CommandResult runCommand(const QString &binaryName, const QStringList &arguments,
                             int timeoutMs = 5000) const;
    QString commandFailureDetail(const CommandResult &result,
                                 const QString &fallback) const;
};
