#include "cli_adapter.h"
#include "workspace_config.h"
#include "workspace_pages.h"

#include <QApplication>
#include <QByteArray>
#include <QButtonGroup>
#include <QComboBox>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QHBoxLayout>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QMainWindow>
#include <QMenu>
#include <QMenuBar>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QSplitter>
#include <QSpinBox>
#include <QStackedWidget>
#include <QStatusBar>
#include <QTableWidget>
#include <QTextStream>
#include <QVBoxLayout>
#include <QWidget>

#include <functional>

namespace {

QString configuredRepoRoot(const QApplication &app) {
    const QStringList args = app.arguments();
    for (int index = 1; index + 1 < args.size(); ++index) {
        if (args.at(index) == "--repo-root") {
            return QDir(args.at(index + 1)).absolutePath();
        }
    }
    return QDir::currentPath();
}

QString configuredConfigPath(const QApplication &app) {
    const QStringList args = app.arguments();
    for (int index = 1; index + 1 < args.size(); ++index) {
        if (args.at(index) == "--config") {
            return args.at(index + 1);
        }
    }
    return {};
}

bool allowUserConfig(const QApplication &app) {
    return !app.arguments().contains("--no-user-config");
}

bool checkRootsOnly(const QApplication &app) {
    return app.arguments().contains("--check-roots");
}

bool smokeProjectCreateSave(const QApplication &app) {
    return app.arguments().contains("--smoke-project-create-save");
}

bool smokePublishTargetStatus(const QApplication &app) {
    return app.arguments().contains("--smoke-publish-target-status");
}

bool smokeRecordingAttach(const QApplication &app) {
    return app.arguments().contains("--smoke-recording-attach");
}

QString optionValue(const QApplication &app, const QString &name) {
    const QStringList args = app.arguments();
    for (int index = 1; index + 1 < args.size(); ++index) {
        if (args.at(index) == name) {
            return args.at(index + 1);
        }
    }
    return {};
}

bool appendBlockedPublishTarget(const QString &projectPath, QString *error) {
    QFile manifest(QDir(projectPath).filePath("project.toml"));
    if (!manifest.open(QIODevice::Append | QIODevice::Text)) {
        if (error != nullptr) {
            *error = "could not open project manifest for append";
        }
        return false;
    }

    QTextStream stream(&manifest);
    stream << "\n";
    stream << "[[publish.targets]]\n";
    stream << "name = \"production\"\n";
    stream << "method = \"rsync\"\n";
    stream << "host = \"missing-host\"\n";
    stream << "identity = \"missing-identity\"\n";
    stream << "remote_path = \"/srv/gemini/smoke\"\n";
    stream << "url = \"gemini://example.invalid\"\n";
    stream << "delete_policy = \"confirm\"\n";
    return true;
}

bool writeFile(const QString &path, const QByteArray &content, QString *error) {
    QFile file(path);
    if (!file.open(QIODevice::WriteOnly | QIODevice::Truncate)) {
        if (error != nullptr) {
            *error = "could not write file: " + path;
        }
        return false;
    }
    if (file.write(content) != content.size()) {
        if (error != nullptr) {
            *error = "short write: " + path;
        }
        return false;
    }
    return true;
}

void appendLe16(QByteArray *bytes, quint16 value) {
    bytes->append(static_cast<char>(value & 0xff));
    bytes->append(static_cast<char>((value >> 8) & 0xff));
}

void appendLe32(QByteArray *bytes, quint32 value) {
    bytes->append(static_cast<char>(value & 0xff));
    bytes->append(static_cast<char>((value >> 8) & 0xff));
    bytes->append(static_cast<char>((value >> 16) & 0xff));
    bytes->append(static_cast<char>((value >> 24) & 0xff));
}

bool writeTestWav(const QString &path, QString *error) {
    const quint32 sampleRate = 48000;
    const quint16 channels = 1;
    const quint16 bitsPerSample = 16;
    const quint32 sampleCount = 4800;
    const quint32 bytesPerSample = bitsPerSample / 8;
    const quint32 dataLen = sampleCount * channels * bytesPerSample;
    const quint32 byteRate = sampleRate * channels * bytesPerSample;
    const quint16 blockAlign = channels * (bitsPerSample / 8);

    QByteArray bytes;
    bytes.append("RIFF", 4);
    appendLe32(&bytes, 36 + dataLen);
    bytes.append("WAVE", 4);
    bytes.append("fmt ", 4);
    appendLe32(&bytes, 16);
    appendLe16(&bytes, 1);
    appendLe16(&bytes, channels);
    appendLe32(&bytes, sampleRate);
    appendLe32(&bytes, byteRate);
    appendLe16(&bytes, blockAlign);
    appendLe16(&bytes, bitsPerSample);
    bytes.append("data", 4);
    appendLe32(&bytes, dataLen);
    bytes.append(QByteArray(static_cast<int>(dataLen), '\0'));

    return writeFile(path, bytes, error);
}

QString readFile(const QString &path, QString *error) {
    QFile file(path);
    if (!file.open(QIODevice::ReadOnly)) {
        if (error != nullptr) {
            *error = "could not read file: " + path;
        }
        return {};
    }
    return QString::fromUtf8(file.readAll());
}

bool comboContains(const QComboBox *combo, const QString &value) {
    for (int index = 0; index < combo->count(); ++index) {
        if (combo->itemText(index) == value) {
            return true;
        }
    }
    return false;
}

int tableRowWithText(const QTableWidget *table, int column, const QString &value) {
    for (int row = 0; row < table->rowCount(); ++row) {
        auto *item = table->item(row, column);
        if (item != nullptr && item->text() == value) {
            return row;
        }
    }
    return -1;
}

int runProjectCreateSaveSmoke(const CliAdapter &adapter, const QApplication &app) {
    QTextStream out(stdout);
    QTextStream err(stderr);

    const QString id = optionValue(app, "--smoke-project-id");
    const QString name = optionValue(app, "--smoke-project-name");
    const QString type = optionValue(app, "--smoke-project-type");
    if (id.isEmpty() || name.isEmpty() || type.isEmpty()) {
        err << "workspace project smoke: id, name, and type are required" << Qt::endl;
        return 2;
    }

    const ProjectCreateResult created = adapter.createProject(id, name, type);
    if (!created.ok) {
        err << "workspace project smoke: create failed: " << created.error << Qt::endl;
        return 1;
    }

    const ProjectTargetResult target =
        adapter.addRemovablePublishTarget(created.projectPath, "export", "publish/export");
    if (!target.ok) {
        err << "workspace project smoke: target failed: " << target.error << Qt::endl;
        return 1;
    }

    ProjectDocument document = adapter.loadProjectDocument(created.projectPath);
    if (!document.ok) {
        err << "workspace project smoke: load failed: " << document.error << Qt::endl;
        return 1;
    }

    const QString editedText = "# " + name + "\n\nSaved from Workspace smoke.\n";
    QString saveError;
    if (!adapter.saveProjectDocument(document, editedText, &saveError)) {
        err << "workspace project smoke: save failed: " << saveError << Qt::endl;
        return 1;
    }

    document = adapter.loadProjectDocument(created.projectPath);
    if (!document.ok || document.text != editedText) {
        err << "workspace project smoke: saved content did not round-trip" << Qt::endl;
        return 1;
    }

    QFile extraContent(QDir(document.contentRootPath).filePath("notes.gmi"));
    if (!extraContent.open(QIODevice::WriteOnly | QIODevice::Text |
                           QIODevice::Truncate)) {
        err << "workspace project smoke: extra content file could not be created"
            << Qt::endl;
        return 1;
    }
    extraContent.write("# Notes\n");
    extraContent.close();

    const QString validation = adapter.projectValidationState(created.projectPath);
    if (validation != "valid") {
        err << "workspace project smoke: validation returned " << validation
            << Qt::endl;
        return 1;
    }

    QWidget *page = createPage(&adapter);
    QApplication::processEvents();
    auto *contentFileFilter = page->findChild<QLineEdit *>("createContentFileFilter");
    auto *contentFiles = page->findChild<QTableWidget *>("createContentFilesTable");
    auto *contentFileDetail =
        page->findChild<QPlainTextEdit *>("createContentFileDetail");
    const int contentIndexRow =
        contentFiles == nullptr ? -1 : tableRowWithText(contentFiles, 0, "index.gmi");
    const int notesRow =
        contentFiles == nullptr ? -1 : tableRowWithText(contentFiles, 0, "notes.gmi");
    if (contentFileFilter == nullptr || contentFiles == nullptr ||
        contentFileDetail == nullptr || contentFiles->rowCount() < 2 ||
        contentIndexRow < 0 || notesRow < 0 ||
        contentFiles->item(contentIndexRow, 2) == nullptr ||
        QDir::cleanPath(contentFiles->item(contentIndexRow, 2)->text()) !=
            QDir::cleanPath(document.contentPath)) {
        err << "workspace project smoke: content file list did not include index.gmi"
            << Qt::endl;
        delete page;
        return 1;
    }
    contentFiles->selectRow(notesRow);
    QApplication::processEvents();
    if (!contentFileDetail->toPlainText().contains("File: notes.gmi") ||
        !contentFileDetail->toPlainText().contains("Editable index: no")) {
        err << "workspace project smoke: content file detail did not describe notes.gmi"
            << Qt::endl;
        delete page;
        return 1;
    }
    contentFileFilter->setText("index");
    QApplication::processEvents();
    if (contentFiles->rowCount() != 1 || contentFiles->item(0, 0) == nullptr ||
        contentFiles->item(0, 0)->text() != "index.gmi" ||
        !contentFileDetail->toPlainText().contains("File: index.gmi") ||
        !contentFileDetail->toPlainText().contains("Editable index: yes")) {
        err << "workspace project smoke: content file filter did not isolate index.gmi"
            << Qt::endl;
        delete page;
        return 1;
    }
    delete page;

    out << "workspace project smoke: created, targeted, saved, and validated "
        << created.projectPath << Qt::endl;
    return 0;
}

int runRecordingAttachSmoke(const CliAdapter &adapter, const QApplication &app) {
    QTextStream out(stdout);
    QTextStream err(stderr);

    const QString id = optionValue(app, "--smoke-project-id");
    const QString name = optionValue(app, "--smoke-project-name");
    const QString type = optionValue(app, "--smoke-project-type");
    if (id.isEmpty() || name.isEmpty() || type.isEmpty()) {
        err << "workspace recording smoke: id, name, and type are required"
            << Qt::endl;
        return 2;
    }

    const ProjectCreateResult created = adapter.createProject(id, name, type);
    if (!created.ok) {
        err << "workspace recording smoke: create failed: " << created.error
            << Qt::endl;
        return 1;
    }
    const ProjectTargetResult target =
        adapter.addRemovablePublishTarget(created.projectPath, "export", "publish/export");
    if (!target.ok) {
        err << "workspace recording smoke: export target failed: " << target.error
            << Qt::endl;
        return 1;
    }

    QDir projectDir(created.projectPath);
    if (!QFileInfo(projectDir.filePath("audio/masters")).isDir() ||
        !QFileInfo(projectDir.filePath("audio/published")).isDir() ||
        !QFileInfo(projectDir.filePath("audio/metadata")).isDir() ||
        !QFileInfo(projectDir.filePath("feeds/feed.xml")).isFile()) {
        err << "workspace recording smoke: audio project defaults were not created"
            << Qt::endl;
        return 1;
    }

    QWidget *page = createPage(&adapter);
    QApplication::processEvents();

    auto *projects = page->findChild<QTableWidget *>("createProjectsTable");
    auto *recordingId = page->findChild<QLineEdit *>("createRecordingId");
    auto *recordingTitle = page->findChild<QLineEdit *>("createRecordingTitle");
    auto *recordingMaster = page->findChild<QLineEdit *>("createRecordingMaster");
    auto *recordingPublished =
        page->findChild<QLineEdit *>("createRecordingPublished");
    auto *recordingFeed = page->findChild<QLineEdit *>("createRecordingFeed");
    auto *recordingEntryId = page->findChild<QLineEdit *>("createRecordingEntryId");
    auto *recordingMime = page->findChild<QLineEdit *>("createRecordingMimeType");
    auto *captureDuration =
        page->findChild<QSpinBox *>("createCaptureDurationSeconds");
    auto *captureInputFormat =
        page->findChild<QComboBox *>("createCaptureInputFormat");
    auto *captureInput = page->findChild<QLineEdit *>("createCaptureInput");
    auto *recordingExportPreset =
        page->findChild<QComboBox *>("createRecordingExportPreset");
    auto *feedUpdated = page->findChild<QLineEdit *>("createFeedEntryUpdated");
    auto *feedSummary = page->findChild<QLineEdit *>("createFeedEntrySummary");
    auto *captureRecording = page->findChild<QPushButton *>("createCaptureRecording");
    auto *exportOpus = page->findChild<QPushButton *>("createExportOpus");
    auto *attach = page->findChild<QPushButton *>("createAttachRecording");
    auto *updateRecording = page->findChild<QPushButton *>("createUpdateRecording");
    auto *prepareFeedEntry = page->findChild<QPushButton *>("createPrepareFeedEntry");
    auto *updateFeedEntry = page->findChild<QPushButton *>("createUpdateFeedEntry");
    auto *validatePublication =
        page->findChild<QPushButton *>("createValidatePublication");
    auto *validateFeedEntry =
        page->findChild<QPushButton *>("createValidateFeedEntry");
    auto *generateFeed = page->findChild<QPushButton *>("createGenerateFeed");
    auto *status = page->findChild<QLabel *>("createRecordingAttachStatus");
    auto *feedStatus = page->findChild<QLabel *>("createFeedEntryStatus");
    auto *details = page->findChild<QPlainTextEdit *>("createRecordingDetails");
    if (projects == nullptr || recordingId == nullptr ||
        recordingTitle == nullptr || recordingMaster == nullptr ||
        recordingPublished == nullptr || recordingFeed == nullptr ||
        recordingEntryId == nullptr || recordingMime == nullptr ||
        captureDuration == nullptr || captureInputFormat == nullptr ||
        captureInput == nullptr || recordingExportPreset == nullptr ||
        feedUpdated == nullptr || feedSummary == nullptr ||
        captureRecording == nullptr || exportOpus == nullptr || attach == nullptr ||
        updateRecording == nullptr || prepareFeedEntry == nullptr ||
        updateFeedEntry == nullptr || validatePublication == nullptr ||
        validateFeedEntry == nullptr || generateFeed == nullptr ||
        status == nullptr || feedStatus == nullptr || details == nullptr) {
        err << "workspace recording smoke: recording widgets were not discoverable"
            << Qt::endl;
        delete page;
        return 1;
    }

    const int projectRow = tableRowWithText(projects, 3, created.projectPath);
    if (projectRow < 0) {
        err << "workspace recording smoke: created project was not listed"
            << Qt::endl;
        delete page;
        return 1;
    }
    projects->selectRow(projectRow);
    QApplication::processEvents();

    recordingId->setText("field-note");
    recordingTitle->setText("Field Note");
    recordingMaster->setText("audio/masters/field-note.wav");
    recordingPublished->setText("audio/published/field-note.opus");
    captureDuration->setValue(1);
    captureInputFormat->setCurrentText("lavfi");
    captureInput->setText("anullsrc=r=48000:cl=mono");
    recordingExportPreset->setCurrentText("voice-standard");
    recordingFeed->setText("feeds/feed.xml");
    recordingEntryId->setText("tag:example.invalid,2026:field-note");
    recordingMime->setText("audio/ogg; codecs=opus");
    feedUpdated->setText("2026-07-19T00:00:00Z");
    feedSummary->setText("Workspace feed entry smoke");
    captureRecording->click();
    QApplication::processEvents();

    const QString masterPath = projectDir.filePath("audio/masters/field-note.wav");
    if (!QFileInfo::exists(masterPath) ||
        !status->text().contains("audio/masters/field-note.wav") ||
        !status->text().contains("ffmpeg") ||
        !details->toPlainText().contains("Captured master: audio/masters/field-note.wav") ||
        !details->toPlainText().contains("Sample rate: 48000")) {
        err << "workspace recording smoke: capture was not reflected"
            << Qt::endl;
        err << "status: " << status->text() << Qt::endl;
        err << "details: " << details->toPlainText() << Qt::endl;
        delete page;
        return 1;
    }

    exportOpus->click();
    QApplication::processEvents();

    const QString publishedPath =
        projectDir.filePath("audio/published/field-note.opus");
    if (!QFileInfo::exists(publishedPath) ||
        !status->text().contains("audio/published/field-note.opus") ||
        !status->text().contains("ffmpeg") ||
        !details->toPlainText().contains("Publication copy: audio/published/field-note.opus") ||
        !details->toPlainText().contains("Engine: ffmpeg")) {
        err << "workspace recording smoke: Opus export was not reflected"
            << Qt::endl;
        err << "status: " << status->text() << Qt::endl;
        err << "details: " << details->toPlainText() << Qt::endl;
        delete page;
        return 1;
    }

    attach->click();
    QApplication::processEvents();

    const QString metadataPath =
        projectDir.filePath("audio/metadata/field-note.toml");
    if (!QFileInfo::exists(metadataPath) ||
        !status->text().contains("audio/metadata/field-note.toml") ||
        !details->toPlainText().contains("Recording: Field Note") ||
        !details->toPlainText().contains("Published: audio/published/field-note.opus")) {
        err << "workspace recording smoke: recording attachment was not reflected"
            << Qt::endl;
        err << "status: " << status->text() << Qt::endl;
        err << "details: " << details->toPlainText() << Qt::endl;
        delete page;
        return 1;
    }

    QString setupError;
    if (!writeTestWav(projectDir.filePath("audio/masters/field-note-revised.wav"),
                      &setupError) ||
        !writeFile(projectDir.filePath("audio/published/field-note-revised.opus"),
                   "revised published", &setupError)) {
        err << "workspace recording smoke: revised audio file setup failed: "
            << setupError << Qt::endl;
        delete page;
        return 1;
    }

    recordingTitle->setText("Field Note Revised");
    recordingMaster->setText("audio/masters/field-note-revised.wav");
    recordingPublished->setText("audio/published/field-note-revised.opus");
    recordingEntryId->setText("tag:example.invalid,2026:field-note-revised");
    updateRecording->click();
    QApplication::processEvents();

    const QString revisedMetadata = readFile(metadataPath, &setupError);
    if (!status->text().contains("Updated: audio/metadata/field-note.toml") ||
        !details->toPlainText().contains("Recording: Field Note Revised") ||
        !details->toPlainText().contains(
            "Published: audio/published/field-note-revised.opus") ||
        !revisedMetadata.contains("id = \"field-note\"") ||
        !revisedMetadata.contains("title = \"Field Note Revised\"") ||
        !revisedMetadata.contains(
            "published = \"audio/published/field-note-revised.opus\"")) {
        err << "workspace recording smoke: recording update was not reflected"
            << Qt::endl;
        err << "status: " << status->text() << Qt::endl;
        err << "details: " << details->toPlainText() << Qt::endl;
        err << "metadata: " << revisedMetadata << Qt::endl;
        delete page;
        return 1;
    }

    prepareFeedEntry->click();
    QApplication::processEvents();
    const QString feedEntryPath =
        projectDir.filePath("feeds/entries/field-note.toml");
    if (!QFileInfo::exists(feedEntryPath) ||
        !feedStatus->text().contains("feeds/entries/field-note.toml") ||
        !feedStatus->text().contains("publication valid") ||
        !feedStatus->text().contains("feed entry valid") ||
        !details->toPlainText().contains("Feed entry: feeds/entries/field-note.toml")) {
        err << "workspace recording smoke: feed entry preparation was not reflected"
            << Qt::endl;
        err << "feed status: " << feedStatus->text() << Qt::endl;
        err << "details: " << details->toPlainText() << Qt::endl;
        delete page;
        return 1;
    }

    feedUpdated->setText("2026-07-20T00:00:00Z");
    feedSummary->setText("Workspace feed entry update smoke");
    updateFeedEntry->click();
    QApplication::processEvents();

    const QString updatedFeedEntry = readFile(feedEntryPath, &setupError);
    if (!feedStatus->text().contains("Updated: feeds/entries/field-note.toml") ||
        !feedStatus->text().contains("publication valid") ||
        !feedStatus->text().contains("feed entry valid") ||
        !updatedFeedEntry.contains("updated = \"2026-07-20T00:00:00Z\"") ||
        !updatedFeedEntry.contains("summary = \"Workspace feed entry update smoke\"") ||
        !details->toPlainText().contains("Feed entry: feeds/entries/field-note.toml")) {
        err << "workspace recording smoke: feed entry update was not reflected"
            << Qt::endl;
        err << "feed status: " << feedStatus->text() << Qt::endl;
        err << "details: " << details->toPlainText() << Qt::endl;
        err << "feed entry: " << updatedFeedEntry << Qt::endl;
        delete page;
        return 1;
    }

    validatePublication->click();
    QApplication::processEvents();
    if (!feedStatus->text().contains("Publication: valid")) {
        err << "workspace recording smoke: publication validation was not reflected"
            << Qt::endl;
        err << "feed status: " << feedStatus->text() << Qt::endl;
        delete page;
        return 1;
    }

    validateFeedEntry->click();
    QApplication::processEvents();
    if (!feedStatus->text().contains("Feed entry: valid")) {
        err << "workspace recording smoke: feed-entry validation was not reflected"
            << Qt::endl;
        err << "feed status: " << feedStatus->text() << Qt::endl;
        delete page;
        return 1;
    }

    generateFeed->click();
    QApplication::processEvents();
    const QString feedPath = projectDir.filePath("feeds/feed.xml");
    if (!QFileInfo::exists(feedPath) ||
        !feedStatus->text().contains("Feed generated: feeds/feed.xml") ||
        !feedStatus->text().contains("1 entries") ||
        !details->toPlainText().contains("Feed: feeds/feed.xml") ||
        !details->toPlainText().contains("Entries: 1")) {
        err << "workspace recording smoke: feed generation was not reflected"
            << Qt::endl;
        err << "feed status: " << feedStatus->text() << Qt::endl;
        err << "details: " << details->toPlainText() << Qt::endl;
        delete page;
        return 1;
    }

    if (!writeFile(projectDir.filePath("feeds/entries/broken.toml"),
                   "[entry]\n"
                   "id = \"tag:example.invalid,2026:broken\"\n"
                   "title = \"Broken\"\n"
                   "updated = \"2026-07-20T00:00:00Z\"\n"
                   "summary = \"Broken summary\"\n"
                   "feed = \"feeds/feed.xml\"\n"
                   "recording = \"missing\"\n\n"
                   "[enclosure]\n"
                   "path = \"audio/published/missing.opus\"\n"
                   "mime_type = \"audio/ogg; codecs=opus\"\n",
                   &setupError)) {
        err << "workspace recording smoke: broken feed-entry setup failed: "
            << setupError << Qt::endl;
        delete page;
        return 1;
    }

    bool handoffCalled = false;
    QWidget *publish = publishPage(&adapter, [&](const QString &projectPath,
                                                 const QString &recordingId) {
        handoffCalled = focusCreateProject(page, projectPath, recordingId);
        return handoffCalled;
    });
    QApplication::processEvents();
    auto *publishFilter = publish->findChild<QLineEdit *>("publishProjectFilter");
    auto *publishProjects = publish->findChild<QTableWidget *>("publishProjectsTable");
    auto *publishStatus = publish->findChild<QLabel *>("publishPreviewStatus");
    auto *publishPlan = publish->findChild<QPlainTextEdit *>("publishPlan");
    auto *publishFeedDiagnostic =
        publish->findChild<QComboBox *>("publishFeedDiagnosticSelector");
    auto *validateFeedDiagnostic =
        publish->findChild<QPushButton *>("publishValidateFeedDiagnostic");
    auto *openFeedDiagnosticInCreate =
        publish->findChild<QPushButton *>("publishOpenFeedDiagnosticInCreate");
    auto *feedDiagnosticDetail =
        publish->findChild<QPlainTextEdit *>("publishFeedDiagnosticDetail");
    if (publishFilter == nullptr || publishProjects == nullptr ||
        publishStatus == nullptr || publishPlan == nullptr ||
        publishFeedDiagnostic == nullptr || validateFeedDiagnostic == nullptr ||
        openFeedDiagnosticInCreate == nullptr || feedDiagnosticDetail == nullptr) {
        err << "workspace recording smoke: publish widgets were not discoverable"
            << Qt::endl;
        delete publish;
        delete page;
        return 1;
    }
    publishFilter->setText(id);
    QApplication::processEvents();
    const int publishRow = tableRowWithText(publishProjects, 2, created.projectPath);
    if (publishRow < 0) {
        err << "workspace recording smoke: generated project was not listed in publish pane"
            << Qt::endl;
        delete publish;
        delete page;
        return 1;
    }
    publishProjects->selectRow(publishRow);
    QApplication::processEvents();
    if (!publishStatus->text().contains("feed entries invalid") ||
        !publishPlan->toPlainText().contains("Feed:") ||
        !publishPlan->toPlainText().contains("Path: feeds/feed.xml") ||
        !publishPlan->toPlainText().contains("Prepared entries: 1") ||
        !publishPlan->toPlainText().contains("Invalid entries: 1") ||
        !publishPlan->toPlainText().contains("Diagnostics:") ||
        !publishPlan->toPlainText().contains("feeds/entries/broken.toml") ||
        !publishPlan->toPlainText().contains("entry.recording_metadata")) {
        err << "workspace recording smoke: publish feed state was not reflected"
            << Qt::endl;
        err << "publish status: " << publishStatus->text() << Qt::endl;
        err << "publish plan: " << publishPlan->toPlainText() << Qt::endl;
        delete publish;
        delete page;
        return 1;
    }
    if (publishFeedDiagnostic->count() != 1 ||
        publishFeedDiagnostic->currentText() != "feeds/entries/broken.toml" ||
        !feedDiagnosticDetail->toPlainText().contains("Feed-entry diagnostic") ||
        !feedDiagnosticDetail->toPlainText().contains("entry.recording_metadata")) {
        err << "workspace recording smoke: publish feed diagnostic selector was not reflected"
            << Qt::endl;
        err << "diagnostic: " << publishFeedDiagnostic->currentText() << Qt::endl;
        err << "detail: " << feedDiagnosticDetail->toPlainText() << Qt::endl;
        delete publish;
        delete page;
        return 1;
    }
    validateFeedDiagnostic->click();
    QApplication::processEvents();
    if (!feedDiagnosticDetail->toPlainText().contains("Feed-entry validation") ||
        !feedDiagnosticDetail->toPlainText().contains("Valid: no") ||
        !feedDiagnosticDetail->toPlainText().contains("missing_feed_entry_field") ||
        !feedDiagnosticDetail->toPlainText().contains("entry.recording_metadata")) {
        err << "workspace recording smoke: feed diagnostic validation detail was not reflected"
            << Qt::endl;
        err << "detail: " << feedDiagnosticDetail->toPlainText() << Qt::endl;
        delete publish;
        delete page;
        return 1;
    }
    openFeedDiagnosticInCreate->click();
    QApplication::processEvents();
    if (!handoffCalled || recordingId->text().trimmed() != "broken" ||
        !feedStatus->text().contains("Loaded from Publish diagnostic: broken") ||
        !feedDiagnosticDetail->toPlainText().contains("Opened in Create") ||
        !feedDiagnosticDetail->toPlainText().contains("feeds/entries/broken.toml")) {
        err << "workspace recording smoke: feed diagnostic handoff was not reflected"
            << Qt::endl;
        err << "recording id: " << recordingId->text() << Qt::endl;
        err << "feed status: " << feedStatus->text() << Qt::endl;
        err << "detail: " << feedDiagnosticDetail->toPlainText() << Qt::endl;
        delete publish;
        delete page;
        return 1;
    }

    delete publish;
    delete page;
    out << "workspace recording smoke: capture, attachment, feed-entry, feed generation, and publish feed state controls succeeded "
        << metadataPath << Qt::endl;
    return 0;
}

int runPublishTargetStatusSmoke(const CliAdapter &adapter, const QApplication &app) {
    QTextStream out(stdout);
    QTextStream err(stderr);

    const QString id = optionValue(app, "--smoke-project-id");
    const QString name = optionValue(app, "--smoke-project-name");
    const QString type = optionValue(app, "--smoke-project-type");
    if (id.isEmpty() || name.isEmpty() || type.isEmpty()) {
        err << "workspace publish smoke: id, name, and type are required" << Qt::endl;
        return 2;
    }

    const ProjectCreateResult created = adapter.createProject(id, name, type);
    if (!created.ok) {
        err << "workspace publish smoke: create failed: " << created.error << Qt::endl;
        return 1;
    }

    const ProjectTargetResult exportTarget =
        adapter.addRemovablePublishTarget(created.projectPath, "export", "publish/export");
    if (!exportTarget.ok) {
        err << "workspace publish smoke: export target failed: " << exportTarget.error
            << Qt::endl;
        return 1;
    }

    const ProjectTargetResult backupTarget =
        adapter.addRemovablePublishTarget(created.projectPath, "backup", "publish/backup");
    if (!backupTarget.ok) {
        err << "workspace publish smoke: backup target failed: " << backupTarget.error
            << Qt::endl;
        return 1;
    }

    QString appendError;
    if (!appendBlockedPublishTarget(created.projectPath, &appendError)) {
        err << "workspace publish smoke: blocked target failed: " << appendError
            << Qt::endl;
        return 1;
    }

    const CompletedHistorySaveResult completedRecord =
        adapter.saveCompletedPublicationHistory(
            created.projectPath, "export", "2026-07-20T00:00:00Z", "completed",
            "passed", false, "Workspace smoke completed record");
    if (!completedRecord.ok) {
        err << "workspace publish smoke: completed history seed failed: "
            << completedRecord.error << Qt::endl;
        return 1;
    }

    QWidget *page = publishPage(&adapter);
    QApplication::processEvents();

    auto *selector = page->findChild<QComboBox *>("publishTargetSelector");
    auto *status = page->findChild<QLabel *>("publishPreviewStatus");
    auto *remoteState = page->findChild<QLineEdit *>("publishRemoteStatePath");
    auto *exportState = page->findChild<QPushButton *>("publishExportRemovableState");
    auto *exportStateStatus =
        page->findChild<QLabel *>("publishRemovableStateExportStatus");
    auto *previewButton = page->findChild<QPushButton *>("publishPreview");
    auto *savePreview = page->findChild<QPushButton *>("publishSavePreview");
    auto *saveStatus = page->findChild<QLabel *>("publishSavePreviewStatus");
    auto *plan = page->findChild<QPlainTextEdit *>("publishPlan");
    auto *validation = page->findChild<QPlainTextEdit *>("publishValidationReport");
    auto *transferIntent = page->findChild<QPlainTextEdit *>("publishTransferIntent");
    auto *removableExecution =
        page->findChild<QPlainTextEdit *>("publishRemovableExecutionPlan");
    auto *historySummary = page->findChild<QPlainTextEdit *>("publishHistorySummary");
    auto *projectFilter = page->findChild<QLineEdit *>("publishProjectFilter");
    auto *savedPreviewFilter = page->findChild<QLineEdit *>("publishSavedPreviewFilter");
    auto *savedPreviews = page->findChild<QTableWidget *>("publishSavedPreviewsTable");
    auto *savedPreviewDetail =
        page->findChild<QPlainTextEdit *>("publishSavedPreviewDetail");
    auto *historyComparison =
        page->findChild<QPlainTextEdit *>("publishHistoryComparison");
    auto *completedHistoryFilter =
        page->findChild<QLineEdit *>("publishCompletedHistoryFilter");
    auto *completedRecords =
        page->findChild<QTableWidget *>("publishCompletedHistoryTable");
    auto *completedRecordDetail =
        page->findChild<QPlainTextEdit *>("publishCompletedHistoryDetail");
    auto *history = page->findChild<QPlainTextEdit *>("publishPlannedHistory");
    auto *projects = page->findChild<QTableWidget *>("publishProjectsTable");
    auto *targetOverview = page->findChild<QTableWidget *>("publishTargetOverviewTable");
    if (selector == nullptr || status == nullptr || remoteState == nullptr ||
        exportState == nullptr || exportStateStatus == nullptr ||
        previewButton == nullptr || savePreview == nullptr || saveStatus == nullptr ||
        plan == nullptr || validation == nullptr || transferIntent == nullptr ||
        removableExecution == nullptr || historySummary == nullptr ||
        projectFilter == nullptr ||
        savedPreviewFilter == nullptr || savedPreviews == nullptr ||
        savedPreviewDetail == nullptr || historyComparison == nullptr ||
        completedHistoryFilter == nullptr || completedRecords == nullptr ||
        completedRecordDetail == nullptr || history == nullptr || projects == nullptr ||
        targetOverview == nullptr) {
        err << "workspace publish smoke: publish widgets were not discoverable"
            << Qt::endl;
        delete page;
        return 1;
    }

    const QString remoteStatePath = QDir(created.projectPath).filePath("remote-state.txt");
    QString remoteStateError;
    if (!writeFile(remoteStatePath, "content/index.gmi\nstale.gmi\n",
                   &remoteStateError)) {
        err << "workspace publish smoke: remote-state setup failed: "
            << remoteStateError << Qt::endl;
        delete page;
        return 1;
    }

    projectFilter->setText("publish");
    QApplication::processEvents();
    if (projects->rowCount() != 1 || projects->item(0, 0) == nullptr ||
        projects->item(0, 0)->text() != name ||
        status->text() != "Preview: ready" ||
        !plan->toPlainText().contains("Target: export")) {
        err << "workspace publish smoke: project filter did not isolate publish project"
            << Qt::endl;
        delete page;
        return 1;
    }

    int projectRow = -1;
    const QString createdPath = QDir::cleanPath(created.projectPath);
    for (int row = 0; row < projects->rowCount(); ++row) {
        auto *item = projects->item(row, 0);
        if (item == nullptr) {
            continue;
        }
        if (QDir::cleanPath(item->data(Qt::UserRole).toString()) == createdPath) {
            projectRow = row;
            break;
        }
    }
    if (projectRow < 0) {
        err << "workspace publish smoke: generated project was not listed" << Qt::endl;
        delete page;
        return 1;
    }

    projects->selectRow(projectRow);
    QApplication::processEvents();
    if (selector->count() != 3 || !comboContains(selector, "export") ||
        !comboContains(selector, "backup") || !comboContains(selector, "production")) {
        err << "workspace publish smoke: target selector did not expose all targets"
            << Qt::endl;
        delete page;
        return 1;
    }
    const int exportOverviewRow = tableRowWithText(targetOverview, 0, "export");
    const int backupOverviewRow = tableRowWithText(targetOverview, 0, "backup");
    const int productionOverviewRow = tableRowWithText(targetOverview, 0, "production");
    if (targetOverview->rowCount() != 3 || exportOverviewRow < 0 ||
        backupOverviewRow < 0 || productionOverviewRow < 0 ||
        targetOverview->item(exportOverviewRow, 1)->text() != "ready" ||
        targetOverview->item(backupOverviewRow, 1)->text() != "ready" ||
        targetOverview->item(productionOverviewRow, 1)->text() != "blocked" ||
        targetOverview->item(exportOverviewRow, 3)->text() != "1" ||
        targetOverview->item(productionOverviewRow, 2)->text() != "rsync") {
        err << "workspace publish smoke: target overview did not summarize targets"
            << Qt::endl;
        delete page;
        return 1;
    }

    if (selector->currentText() != "export" ||
        status->text() != "Preview: ready" ||
        !plan->toPlainText().contains("Target: export") ||
        !validation->toPlainText().contains("Valid: yes") ||
        !validation->toPlainText().contains("Blocked: no") ||
        !transferIntent->toPlainText().contains("Execution ready: yes") ||
        !transferIntent->toPlainText().contains("Target: export") ||
        !transferIntent->toPlainText().contains("Completed history directory:") ||
        !removableExecution->toPlainText().contains("Execution ready: yes") ||
        !removableExecution->toPlainText().contains("Target: export") ||
        !removableExecution->toPlainText().contains("Destination root:") ||
        !removableExecution->toPlainText().contains("content/index.gmi") ||
        !removableExecution->toPlainText().contains("destination:") ||
        !historySummary->toPlainText().contains("planned-upload: 1") ||
        !history->toPlainText().contains("transfer_result = \"planned\"") ||
        !history->toPlainText().contains("target = \"export\"")) {
        err << "workspace publish smoke: export preview was not ready" << Qt::endl;
        delete page;
        return 1;
    }
    if (!historyComparison->toPlainText().startsWith("Comparison:")) {
        err << "workspace publish smoke: saved preview comparison was not updated"
            << Qt::endl;
        delete page;
        return 1;
    }

    const QString removableDestination =
        QDir(created.projectPath).filePath("publish/export/content");
    if (!QDir().mkpath(removableDestination)) {
        err << "workspace publish smoke: removable destination setup failed"
            << Qt::endl;
        delete page;
        return 1;
    }
    QString removableStateSeedError;
    if (!writeFile(QDir(removableDestination).filePath("index.gmi"),
                   "Seeded removable destination\n", &removableStateSeedError)) {
        err << "workspace publish smoke: removable destination seed failed: "
            << removableStateSeedError << Qt::endl;
        delete page;
        return 1;
    }

    exportState->click();
    QApplication::processEvents();
    const QString exportedStatePath = remoteState->text().trimmed();
    const bool exportedUnderProject =
        QDir::cleanPath(exportedStatePath)
            .startsWith(QDir::cleanPath(created.projectPath + "/history/previews/"));
    if (!exportStateStatus->text().startsWith("State exported:") ||
        !exportStateStatus->text().contains("(1 paths)") ||
        exportedStatePath.isEmpty() || !QFileInfo::exists(exportedStatePath) ||
        !exportedUnderProject ||
        !plan->toPlainText().contains("Source: " + exportedStatePath) ||
        !plan->toPlainText().contains("Remote paths: 1") ||
        !plan->toPlainText().contains("Skip:") ||
        !plan->toPlainText().contains("content/index.gmi")) {
        err << "workspace publish smoke: removable state export did not feed comparison"
            << Qt::endl;
        err << "state status: " << exportStateStatus->text() << Qt::endl;
        err << "remote state: " << exportedStatePath << Qt::endl;
        err << "publish plan: " << plan->toPlainText() << Qt::endl;
        delete page;
        return 1;
    }

    if (completedRecords->rowCount() < 1 || completedRecords->item(0, 0) == nullptr ||
        !completedRecords->item(0, 0)->text().contains("export") ||
        !completedRecordDetail->toPlainText().contains("export-completed.toml") ||
        !completedRecordDetail->toPlainText().contains("transfer_result = \"completed\"") ||
        !completedRecordDetail->toPlainText().contains(
            "verification_result = \"passed\"") ||
        !completedRecordDetail->toPlainText().contains(
            "Workspace smoke completed record")) {
        err << "workspace publish smoke: completed history record was not listed"
            << Qt::endl;
        err << "completed detail: " << completedRecordDetail->toPlainText()
            << Qt::endl;
        delete page;
        return 1;
    }

    targetOverview->selectRow(backupOverviewRow);
    QApplication::processEvents();
    if (selector->currentText() != "backup" ||
        status->text() != "Preview: ready" ||
        !plan->toPlainText().contains("Target: backup") ||
        !validation->toPlainText().contains("Target: backup") ||
        !validation->toPlainText().contains("Valid: yes") ||
        !transferIntent->toPlainText().contains("Target: backup") ||
        !transferIntent->toPlainText().contains("Execution ready: yes") ||
        !removableExecution->toPlainText().contains("Target: backup") ||
        !removableExecution->toPlainText().contains("Execution ready: yes") ||
        !historySummary->toPlainText().contains("Target: backup") ||
        !history->toPlainText().contains("target = \"backup\"")) {
        err << "workspace publish smoke: target overview selection did not drive preview"
            << Qt::endl;
        delete page;
        return 1;
    }

    selector->setCurrentText("backup");
    QApplication::processEvents();
    if (status->text() != "Preview: ready" ||
        !plan->toPlainText().contains("Target: backup") ||
        !transferIntent->toPlainText().contains("Target: backup") ||
        !transferIntent->toPlainText().contains("Execution ready: yes") ||
        !removableExecution->toPlainText().contains("Target: backup") ||
        !removableExecution->toPlainText().contains("Execution ready: yes") ||
        !historySummary->toPlainText().contains("Target: backup") ||
        !history->toPlainText().contains("target = \"backup\"")) {
        err << "workspace publish smoke: backup preview was not ready" << Qt::endl;
        delete page;
        return 1;
    }

    selector->setCurrentText("production");
    QApplication::processEvents();
    if (status->text() != "Preview: blocked" ||
        !plan->toPlainText().contains("Target: production") ||
        !plan->toPlainText().contains("Blocked: yes") ||
        !validation->toPlainText().contains("Target: production") ||
        !validation->toPlainText().contains("Valid: no") ||
        !validation->toPlainText().contains("host_missing") ||
        !validation->toPlainText().contains("identity_missing") ||
        !transferIntent->toPlainText().contains("Execution ready: no") ||
        !transferIntent->toPlainText().contains("host_missing") ||
        !transferIntent->toPlainText().contains("identity_missing") ||
        !removableExecution->toPlainText().contains("Execution ready: no") ||
        !removableExecution->toPlainText().contains("unsupported_executor_method") ||
        !historySummary->toPlainText().contains("Target: production") ||
        !historySummary->toPlainText().contains("Verification: not-run") ||
        !history->toPlainText().contains("target = \"production\"") ||
        !history->toPlainText().contains("verification_result = \"not-run\"")) {
        err << "workspace publish smoke: production preview was not blocked"
            << Qt::endl;
        delete page;
        return 1;
    }

    remoteState->setText(remoteStatePath);
    selector->setCurrentText("production");
    QApplication::processEvents();
    previewButton->click();
    QApplication::processEvents();
    if (!plan->toPlainText().contains("Comparison:") ||
        !plan->toPlainText().contains("Source: " + remoteStatePath) ||
        !plan->toPlainText().contains("Remote paths: 2") ||
        !plan->toPlainText().contains("Delete:") ||
        !plan->toPlainText().contains("stale.gmi") ||
        !plan->toPlainText().contains("Skip:") ||
        !plan->toPlainText().contains("content/index.gmi") ||
        !transferIntent->toPlainText().contains("Source: " + remoteStatePath) ||
        !transferIntent->toPlainText().contains("Remote paths: 2") ||
        !transferIntent->toPlainText().contains("stale.gmi") ||
        !removableExecution->toPlainText().contains("delete_execution_not_supported") ||
        !removableExecution->toPlainText().contains("stale.gmi")) {
        err << "workspace publish smoke: remote-state comparison was not reflected"
            << Qt::endl;
        err << "publish plan: " << plan->toPlainText() << Qt::endl;
        err << "transfer intent: " << transferIntent->toPlainText() << Qt::endl;
        err << "removable execution: " << removableExecution->toPlainText()
            << Qt::endl;
        delete page;
        return 1;
    }

    savePreview->click();
    QApplication::processEvents();
    const QString savePrefix = "Saved: ";
    if (!saveStatus->text().startsWith(savePrefix)) {
        err << "workspace publish smoke: planned history preview was not saved"
            << Qt::endl;
        delete page;
        return 1;
    }
    const QString savedPath = saveStatus->text().mid(savePrefix.size());
    if (!QFileInfo::exists(savedPath) ||
        !QDir::cleanPath(savedPath).startsWith(
            QDir::cleanPath(created.projectPath + "/history/previews/"))) {
        err << "workspace publish smoke: saved preview path was outside project"
            << Qt::endl;
        delete page;
        return 1;
    }
    const QString savedFilename = QFileInfo(savedPath).fileName();
    if (savedPreviews->rowCount() < 1 || savedPreviews->item(0, 0) == nullptr ||
        savedPreviews->item(0, 0)->text() != savedFilename) {
        err << "workspace publish smoke: saved preview was not listed"
            << Qt::endl;
        delete page;
        return 1;
    }
    if (!savedPreviewDetail->toPlainText().contains(savedFilename) ||
        !savedPreviewDetail->toPlainText().contains(
            "transfer_result = \"planned\"")) {
        err << "workspace publish smoke: saved preview detail was not loaded"
            << Qt::endl;
        delete page;
        return 1;
    }

    selector->setCurrentText("backup");
    QApplication::processEvents();
    savePreview->click();
    QApplication::processEvents();
    if (savedPreviews->rowCount() < 2 || savedPreviews->currentRow() < 0 ||
        savedPreviews->item(savedPreviews->currentRow(), 0) == nullptr ||
        QDir::cleanPath(savedPreviews->item(savedPreviews->currentRow(), 0)
                            ->data(Qt::UserRole)
                            .toString()) != QDir::cleanPath(savedPath)) {
        err << "workspace publish smoke: saved preview selection was not preserved"
            << Qt::endl;
        delete page;
        return 1;
    }
    if (!historyComparison->toPlainText().contains("differs") ||
        !historyComparison->toPlainText().contains("Generated target: target = \"backup\"") ||
        !historyComparison->toPlainText().contains("Saved target: target = \"production\"")) {
        err << "workspace publish smoke: saved preview comparison did not report target drift"
            << Qt::endl;
        delete page;
        return 1;
    }

    savedPreviewFilter->setText("backup");
    QApplication::processEvents();
    if (savedPreviews->rowCount() != 1 || savedPreviews->item(0, 0) == nullptr ||
        !savedPreviews->item(0, 0)->text().contains("backup") ||
        !savedPreviewDetail->toPlainText().contains("target = \"backup\"")) {
        err << "workspace publish smoke: saved preview filter did not isolate backup"
            << Qt::endl;
        delete page;
        return 1;
    }

    completedHistoryFilter->setText("export");
    QApplication::processEvents();
    if (completedRecords->rowCount() != 1 ||
        completedRecords->item(0, 0) == nullptr ||
        !completedRecords->item(0, 0)->text().contains("export") ||
        !completedRecordDetail->toPlainText().contains("target = \"export\"")) {
        err << "workspace publish smoke: completed history filter did not isolate export"
            << Qt::endl;
        err << "completed detail: " << completedRecordDetail->toPlainText()
            << Qt::endl;
        delete page;
        return 1;
    }

    delete page;
    out << "workspace publish smoke: target selector and status transitions succeeded"
        << Qt::endl;
    return 0;
}

void addMenus(QMainWindow &window) {
    const QStringList menus = {"System", "Project", "Publish", "Network",
                               "Audio", "Window", "Help"};
    for (const auto &name : menus) {
        auto *menu = window.menuBar()->addMenu(name);
        auto *placeholder = menu->addAction(name);
        placeholder->setEnabled(false);
    }
}

void setApplicationStyle(QApplication &app) {
    app.setStyleSheet(R"(
        QMainWindow {
            background: #f5f6f2;
        }
        QMenuBar, QStatusBar {
            background: #e5e8df;
            border: 1px solid #9da69a;
        }
        QListWidget {
            background: #edf0e9;
            border: 1px solid #9da69a;
            outline: 0;
        }
        QListWidget::item {
            min-height: 28px;
            padding: 4px 8px;
        }
        QListWidget::item:selected {
            background: #284b63;
            color: #ffffff;
        }
        QPushButton {
            background: #f8f9f4;
            border: 1px solid #8c968a;
            padding: 5px 12px;
            min-width: 80px;
        }
        QPushButton:checked {
            background: #284b63;
            color: #ffffff;
        }
        QTableWidget, QPlainTextEdit, QTextBrowser {
            background: #ffffff;
            border: 1px solid #9da69a;
            selection-background-color: #284b63;
        }
        QHeaderView::section {
            background: #d9ded3;
            border: 1px solid #9da69a;
            padding: 4px;
        }
        QLabel#sectionLabel {
            color: #17202a;
            font-size: 18px;
            font-weight: 600;
        }
    )");
}

} // namespace

int main(int argc, char *argv[]) {
    QApplication app(argc, argv);
    app.setOrganizationName("WaystoneOS");
    app.setApplicationName("Waystone Workspace");
    const QString repoRoot = configuredRepoRoot(app);
    QString configWarning;
    const WorkspaceConfig workspaceConfig =
        WorkspaceConfig::load(repoRoot, configuredConfigPath(app), allowUserConfig(app),
                              &configWarning);
    if (checkRootsOnly(app)) {
        QTextStream out(stdout);
        QTextStream err(stderr);
        if (!configWarning.isEmpty()) {
            err << "warning: " << configWarning << Qt::endl;
        }

        out << "config source: " << workspaceConfig.configSource << Qt::endl;
        if (!workspaceConfig.configPath.isEmpty()) {
            out << "config path: " << workspaceConfig.configPath << Qt::endl;
        }

        const QStringList missingRoots = workspaceConfig.missingRootMessages();
        if (missingRoots.isEmpty()) {
            out << "roots: ok" << Qt::endl;
            return 0;
        }

        for (const auto &message : missingRoots) {
            err << message << Qt::endl;
        }
        return 2;
    }

    const CliAdapter adapter(workspaceConfig);
    if (smokeProjectCreateSave(app)) {
        return runProjectCreateSaveSmoke(adapter, app);
    }
    if (smokePublishTargetStatus(app)) {
        return runPublishTargetStatusSmoke(adapter, app);
    }
    if (smokeRecordingAttach(app)) {
        return runRecordingAttachSmoke(adapter, app);
    }

    setApplicationStyle(app);

    QMainWindow window;
    window.setWindowTitle("Waystone Workspace");
    window.resize(1100, 680);
    window.setMinimumSize(960, 600);
    addMenus(window);

    auto *root = new QWidget;
    auto *rootLayout = new QVBoxLayout(root);
    rootLayout->setContentsMargins(8, 8, 8, 8);
    rootLayout->setSpacing(8);

    auto *selector = new QWidget;
    auto *selectorLayout = new QHBoxLayout(selector);
    selectorLayout->setContentsMargins(0, 0, 0, 0);
    selectorLayout->setSpacing(6);

    auto *buttonGroup = new QButtonGroup(selector);
    buttonGroup->setExclusive(true);
    const QStringList workspaces = {"Explore", "Create", "Publish", "Operate"};
    const QStringList workspaceToolTips = {
        "Check configured local roots and workspace settings.",
        "Create projects and edit project content. Recordings are optional.",
        "Preview publication readiness and local history. This does not publish.",
        "Inspect local host and identity metadata."
    };
    for (int index = 0; index < workspaces.size(); ++index) {
        auto *button = new QPushButton(workspaces.at(index));
        button->setCheckable(true);
        button->setToolTip(workspaceToolTips.at(index));
        if (index == 0) {
            button->setChecked(true);
        }
        buttonGroup->addButton(button, index);
        selectorLayout->addWidget(button);
    }
    selectorLayout->addStretch();
    rootLayout->addWidget(selector);

    auto *splitter = new QSplitter(Qt::Horizontal);
    auto *navigation = new QListWidget;
    const QList<QPair<QString, QString>> navigationItems = {
        {"Explore", "Check configured local roots and workspace settings."},
        {"Write", "Open Create for project authoring."},
        {"Listen", "Open Create. Listening features are not implemented yet."},
        {"Record", "Open Create > Recordings for optional audio metadata work."},
        {"Publish", "Open Publish for read-only publication preview and history."},
        {"Host", "Open Explore. Host connections are still local metadata."},
        {"Connect", "Open Explore. Network connection features are deferred."},
        {"Learn", "Open Explore. Learning surfaces are deferred."},
        {"Hosts", "Open Operate for local host metadata inspection."},
        {"Services", "Open Operate for local service boundary status."},
        {"Transfers", "Open Operate. Remote transfer execution is deferred."},
        {"Terminal", "Open Operate. Embedded terminal is deferred."},
    };
    for (const auto &item : navigationItems) {
        auto *entry = new QListWidgetItem(item.first, navigation);
        entry->setToolTip(item.second);
    }
    navigation->setCurrentRow(0);
    navigation->setMinimumWidth(150);
    navigation->setMaximumWidth(190);
    splitter->addWidget(navigation);

    auto *pages = new QStackedWidget;
    std::function<void(int)> setWorkspace;
    pages->addWidget(explorePage(workspaceConfig));
    QWidget *create = createPage(&adapter);
    pages->addWidget(create);
    pages->addWidget(publishPage(&adapter, [&](const QString &projectPath,
                                               const QString &recordingId) {
        if (focusCreateProject(create, projectPath, recordingId) && setWorkspace) {
            navigation->setCurrentRow(1);
            setWorkspace(1);
            return true;
        }
        return false;
    }));
    pages->addWidget(operatePage(&adapter));
    splitter->addWidget(pages);
    splitter->setStretchFactor(1, 1);
    rootLayout->addWidget(splitter, 1);

    const QString statusSuffix =
        configWarning.isEmpty() ? QString() : "   " + configWarning;
    setWorkspace = [&](int index) {
        pages->setCurrentIndex(index);
        buttonGroup->button(index)->setChecked(true);
        window.statusBar()->showMessage(
            workspaces.at(index) +
            "   Audio: Idle   Network: Offline   Project: None" + statusSuffix);
    };

    QObject::connect(buttonGroup, &QButtonGroup::idClicked, [&](int index) {
        setWorkspace(index);
    });

    QObject::connect(navigation, &QListWidget::currentRowChanged, [&](int row) {
        if (row == 0 || (row >= 5 && row <= 7)) {
            setWorkspace(0);
        } else if (row >= 1 && row <= 3) {
            setWorkspace(1);
        } else if (row == 4) {
            setWorkspace(2);
        } else if (row >= 8) {
            setWorkspace(3);
        } else {
            setWorkspace(0);
        }
    });

    window.setCentralWidget(root);
    setWorkspace(0);
    window.show();

    return app.exec();
}
