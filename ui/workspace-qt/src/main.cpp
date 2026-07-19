#include "cli_adapter.h"
#include "workspace_config.h"
#include "workspace_pages.h"

#include <QApplication>
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
#include <QStackedWidget>
#include <QStatusBar>
#include <QTableWidget>
#include <QTextStream>
#include <QVBoxLayout>
#include <QWidget>

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

    QDir projectDir(created.projectPath);
    if (!QFileInfo(projectDir.filePath("audio/masters")).isDir() ||
        !QFileInfo(projectDir.filePath("audio/published")).isDir() ||
        !QFileInfo(projectDir.filePath("audio/metadata")).isDir() ||
        !QFileInfo(projectDir.filePath("feeds/feed.xml")).isFile()) {
        err << "workspace recording smoke: audio project defaults were not created"
            << Qt::endl;
        return 1;
    }

    QString setupError;
    if (!writeFile(projectDir.filePath("audio/masters/field-note.flac"), "master",
                   &setupError) ||
        !writeFile(projectDir.filePath("audio/published/field-note.opus"),
                   "published", &setupError)) {
        err << "workspace recording smoke: audio file setup failed: " << setupError
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
    auto *attach = page->findChild<QPushButton *>("createAttachRecording");
    auto *status = page->findChild<QLabel *>("createRecordingAttachStatus");
    auto *details = page->findChild<QPlainTextEdit *>("createRecordingDetails");
    if (projects == nullptr || recordingId == nullptr ||
        recordingTitle == nullptr || recordingMaster == nullptr ||
        recordingPublished == nullptr || recordingFeed == nullptr ||
        recordingEntryId == nullptr || recordingMime == nullptr ||
        attach == nullptr || status == nullptr || details == nullptr) {
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
    recordingMaster->setText("audio/masters/field-note.flac");
    recordingPublished->setText("audio/published/field-note.opus");
    recordingFeed->setText("feeds/feed.xml");
    recordingEntryId->setText("tag:example.invalid,2026:field-note");
    recordingMime->setText("audio/ogg; codecs=opus");
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

    delete page;
    out << "workspace recording smoke: attachment controls succeeded "
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

    QWidget *page = publishPage(&adapter);
    QApplication::processEvents();

    auto *selector = page->findChild<QComboBox *>("publishTargetSelector");
    auto *status = page->findChild<QLabel *>("publishPreviewStatus");
    auto *savePreview = page->findChild<QPushButton *>("publishSavePreview");
    auto *saveStatus = page->findChild<QLabel *>("publishSavePreviewStatus");
    auto *plan = page->findChild<QPlainTextEdit *>("publishPlan");
    auto *historySummary = page->findChild<QPlainTextEdit *>("publishHistorySummary");
    auto *projectFilter = page->findChild<QLineEdit *>("publishProjectFilter");
    auto *savedPreviewFilter = page->findChild<QLineEdit *>("publishSavedPreviewFilter");
    auto *savedPreviews = page->findChild<QTableWidget *>("publishSavedPreviewsTable");
    auto *savedPreviewDetail =
        page->findChild<QPlainTextEdit *>("publishSavedPreviewDetail");
    auto *historyComparison =
        page->findChild<QPlainTextEdit *>("publishHistoryComparison");
    auto *history = page->findChild<QPlainTextEdit *>("publishPlannedHistory");
    auto *projects = page->findChild<QTableWidget *>("publishProjectsTable");
    auto *targetOverview = page->findChild<QTableWidget *>("publishTargetOverviewTable");
    if (selector == nullptr || status == nullptr || savePreview == nullptr ||
        saveStatus == nullptr || plan == nullptr || historySummary == nullptr ||
        projectFilter == nullptr || savedPreviewFilter == nullptr ||
        savedPreviews == nullptr || savedPreviewDetail == nullptr ||
        historyComparison == nullptr || history == nullptr ||
        projects == nullptr || targetOverview == nullptr) {
        err << "workspace publish smoke: publish widgets were not discoverable"
            << Qt::endl;
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

    targetOverview->selectRow(backupOverviewRow);
    QApplication::processEvents();
    if (selector->currentText() != "backup" ||
        status->text() != "Preview: ready" ||
        !plan->toPlainText().contains("Target: backup") ||
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
        !historySummary->toPlainText().contains("Target: production") ||
        !historySummary->toPlainText().contains("Verification: not-run") ||
        !history->toPlainText().contains("target = \"production\"") ||
        !history->toPlainText().contains("verification_result = \"not-run\"")) {
        err << "workspace publish smoke: production preview was not blocked"
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
    for (int index = 0; index < workspaces.size(); ++index) {
        auto *button = new QPushButton(workspaces.at(index));
        button->setCheckable(true);
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
    navigation->addItems({"Explore", "Write", "Listen", "Record", "Publish", "Host",
                          "Connect", "Learn", "Hosts", "Services", "Transfers",
                          "Terminal"});
    navigation->setCurrentRow(0);
    navigation->setMinimumWidth(150);
    navigation->setMaximumWidth(190);
    splitter->addWidget(navigation);

    auto *pages = new QStackedWidget;
    pages->addWidget(explorePage(workspaceConfig));
    pages->addWidget(createPage(&adapter));
    pages->addWidget(publishPage(&adapter));
    pages->addWidget(operatePage(&adapter));
    splitter->addWidget(pages);
    splitter->setStretchFactor(1, 1);
    rootLayout->addWidget(splitter, 1);

    const QString statusSuffix =
        configWarning.isEmpty() ? QString() : "   " + configWarning;
    auto setWorkspace = [&](int index) {
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
