#include "workspace_pages.h"

#include "cli_adapter.h"
#include "workspace_config.h"

#include <QAbstractItemView>
#include <QComboBox>
#include <QDir>
#include <QFileDialog>
#include <QFileInfo>
#include <QFormLayout>
#include <QFrame>
#include <QHBoxLayout>
#include <QHeaderView>
#include <QLabel>
#include <QLineEdit>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QSplitter>
#include <QTableWidget>
#include <QTableWidgetItem>
#include <QTextBrowser>
#include <QVBoxLayout>
#include <QWidget>

#include <memory>

namespace {

QLabel *sectionLabel(const QString &text) {
    auto *label = new QLabel(text);
    label->setObjectName("sectionLabel");
    return label;
}

QFrame *separator() {
    auto *line = new QFrame;
    line->setFrameShape(QFrame::HLine);
    line->setFrameShadow(QFrame::Plain);
    return line;
}

QTableWidget *table(const QStringList &headers, const QList<QStringList> &rows) {
    auto *table = new QTableWidget(rows.size(), headers.size());
    table->setHorizontalHeaderLabels(headers);
    table->verticalHeader()->setVisible(false);
    table->horizontalHeader()->setStretchLastSection(true);
    table->setEditTriggers(QAbstractItemView::NoEditTriggers);
    table->setSelectionBehavior(QAbstractItemView::SelectRows);
    table->setAlternatingRowColors(true);

    for (int row = 0; row < rows.size(); ++row) {
        const auto &values = rows.at(row);
        for (int column = 0; column < values.size(); ++column) {
            table->setItem(row, column, new QTableWidgetItem(values.at(column)));
        }
    }

    return table;
}

QString rootStatus(const QString &path) {
    return QFileInfo::exists(path) ? "available" : "missing";
}

QString renderGemtextPreview(const QString &text) {
    QString html =
        "<html><body style=\"font-family: sans-serif; font-size: 12pt; color: #17202a;\">";
    bool preformatted = false;
    const QStringList lines = text.split('\n');

    for (const QString &line : lines) {
        const QString escaped = line.toHtmlEscaped();
        if (line.startsWith("```")) {
            if (preformatted) {
                html += "</pre>";
            } else {
                html += "<pre style=\"background:#edf0e9; border:1px solid #9da69a; padding:6px;\">";
            }
            preformatted = !preformatted;
            continue;
        }

        if (preformatted) {
            html += escaped + "\n";
            continue;
        }

        if (line.trimmed().isEmpty()) {
            html += "<br>";
        } else if (line.startsWith("### ")) {
            html += "<h3>" + line.mid(4).toHtmlEscaped() + "</h3>";
        } else if (line.startsWith("## ")) {
            html += "<h2>" + line.mid(3).toHtmlEscaped() + "</h2>";
        } else if (line.startsWith("# ")) {
            html += "<h1>" + line.mid(2).toHtmlEscaped() + "</h1>";
        } else if (line.startsWith("=>")) {
            html += "<p style=\"font-family: monospace;\">=> " +
                    line.mid(2).trimmed().toHtmlEscaped() + "</p>";
        } else if (line.startsWith("* ")) {
            html += "<p>&bull; " + line.mid(2).toHtmlEscaped() + "</p>";
        } else if (line.startsWith(">")) {
            html += "<blockquote>" + line.mid(1).trimmed().toHtmlEscaped() +
                    "</blockquote>";
        } else {
            html += "<p>" + escaped + "</p>";
        }
    }

    if (preformatted) {
        html += "</pre>";
    }

    html += "</body></html>";
    return html;
}

QWidget *rootEditorRow(QWidget *parent, const QString &label, QLineEdit *edit) {
    auto *row = new QWidget(parent);
    auto *layout = new QHBoxLayout(row);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(6);

    auto *browse = new QPushButton("Browse", row);
    layout->addWidget(edit, 1);
    layout->addWidget(browse);

    QObject::connect(browse, &QPushButton::clicked, [=]() {
        const QString selected =
            QFileDialog::getExistingDirectory(parent, label, edit->text());
        if (!selected.isEmpty()) {
            edit->setText(QDir::cleanPath(selected));
        }
    });

    return row;
}

void populateProjectTable(QTableWidget *projectsTable, QPlainTextEdit *details,
                          const CliAdapter *adapter) {
    QString error;
    const QList<ProjectSummary> projects = adapter->listProjects(&error);

    projectsTable->setRowCount(projects.size());
    for (int row = 0; row < projects.size(); ++row) {
        const ProjectSummary &project = projects.at(row);
        auto *name = new QTableWidgetItem(project.name);
        name->setData(Qt::UserRole, project.path);
        projectsTable->setItem(row, 0, name);
        projectsTable->setItem(row, 1, new QTableWidgetItem(project.type));
        projectsTable->setItem(row, 2, new QTableWidgetItem(project.validation));
        projectsTable->setItem(row, 3, new QTableWidgetItem(project.path));
    }

    if (!error.isEmpty()) {
        details->setPlainText("Project list failed\n" + error);
        return;
    }

    if (projects.isEmpty()) {
        details->setPlainText("No projects found");
        return;
    }

    projectsTable->selectRow(0);
    details->setPlainText(adapter->inspectProject(projects.at(0).path));
}

bool selectProjectByPath(QTableWidget *projectsTable, const QString &path) {
    const QString wanted = QDir::cleanPath(path);
    for (int row = 0; row < projectsTable->rowCount(); ++row) {
        auto *item = projectsTable->item(row, 0);
        if (item == nullptr) {
            continue;
        }

        const QString itemPath =
            QDir::cleanPath(item->data(Qt::UserRole).toString());
        if (itemPath == wanted) {
            projectsTable->selectRow(row);
            projectsTable->scrollToItem(item);
            return true;
        }
    }

    return false;
}

void populateRecordingTable(QTableWidget *recordingsTable, QPlainTextEdit *details,
                            const CliAdapter *adapter) {
    QString error;
    const QList<RecordingSummary> recordings = adapter->listRecordings(&error);

    recordingsTable->setRowCount(recordings.size());
    for (int row = 0; row < recordings.size(); ++row) {
        const RecordingSummary &recording = recordings.at(row);
        auto *title = new QTableWidgetItem(recording.title);
        title->setData(Qt::UserRole, recording.path);
        recordingsTable->setItem(row, 0, title);
        recordingsTable->setItem(row, 1, new QTableWidgetItem(recording.id));
        recordingsTable->setItem(row, 2, new QTableWidgetItem(recording.validation));
        recordingsTable->setItem(row, 3, new QTableWidgetItem(recording.playable));
        recordingsTable->setItem(row, 4, new QTableWidgetItem(recording.path));
    }

    if (!error.isEmpty()) {
        details->setPlainText("Recording list failed\n" + error);
        return;
    }

    if (recordings.isEmpty()) {
        details->setPlainText("No recordings found");
        return;
    }

    recordingsTable->selectRow(0);
    details->setPlainText(adapter->inspectRecording(recordings.at(0).path));
}

QString defaultPublishTarget(const ProjectSummary &project) {
    if (project.id == "ssh-capsule") {
        return "production";
    }
    return "export";
}

QString renderPublishPreview(const PublishPreview &preview) {
    if (!preview.ok) {
        return "Dry-run preview failed\n" + preview.error;
    }

    QString text;
    text += "Project: " + preview.project + "\n";
    text += "Target: " + preview.target + "\n";
    text += "Method: " + preview.method + "\n";
    text += "Destination: " + preview.destination + "\n";
    text += "Blocked: " + QString(preview.blocked ? "yes" : "no") + "\n";
    text += "Host: " + preview.hostResolution + "\n";
    text += "Identity: " + preview.identityResolution + "\n\n";
    text += "Uploads:\n";
    if (preview.uploads.isEmpty()) {
        text += "  none\n";
    } else {
        for (const auto &upload : preview.uploads) {
            text += "  " + upload + "\n";
        }
    }

    text += "\nVerification:\n";
    if (preview.verificationChecks.isEmpty()) {
        text += "  none\n";
    } else {
        for (const auto &check : preview.verificationChecks) {
            text += "  " + check + "\n";
        }
    }

    text += "\nConfirmations:\n";
    if (preview.confirmations.isEmpty()) {
        text += "  none\n";
    } else {
        for (const auto &confirmation : preview.confirmations) {
            text += "  " + confirmation + "\n";
        }
    }

    return text;
}

void populatePublishTable(QTableWidget *projectsTable, QLineEdit *target,
                          QPlainTextEdit *plan, const CliAdapter *adapter) {
    QString error;
    const QList<ProjectSummary> projects = adapter->listProjects(&error);

    projectsTable->setRowCount(projects.size());
    for (int row = 0; row < projects.size(); ++row) {
        const ProjectSummary &project = projects.at(row);
        const QString targetName = defaultPublishTarget(project);
        auto *name = new QTableWidgetItem(project.name);
        name->setData(Qt::UserRole, project.path);
        name->setData(Qt::UserRole + 1, targetName);
        projectsTable->setItem(row, 0, name);
        projectsTable->setItem(row, 1, new QTableWidgetItem(targetName));
        projectsTable->setItem(row, 2, new QTableWidgetItem(project.path));
    }

    if (!error.isEmpty()) {
        plan->setPlainText("Project list failed\n" + error);
        return;
    }

    if (projects.isEmpty()) {
        target->clear();
        plan->setPlainText("No projects found");
        return;
    }

    projectsTable->selectRow(0);
    target->setText(defaultPublishTarget(projects.at(0)));
    plan->setPlainText(renderPublishPreview(
        adapter->previewPublication(projects.at(0).path, target->text())));
}

void populateHostTable(QTableWidget *hostsTable, QPlainTextEdit *details,
                       const CliAdapter *adapter) {
    QString error;
    const QList<HostSummary> hosts = adapter->listHosts(&error);

    hostsTable->setRowCount(hosts.size());
    for (int row = 0; row < hosts.size(); ++row) {
        const HostSummary &host = hosts.at(row);
        auto *name = new QTableWidgetItem(host.displayName);
        name->setData(Qt::UserRole, host.path);
        hostsTable->setItem(row, 0, name);
        hostsTable->setItem(row, 1, new QTableWidgetItem(host.address));
        hostsTable->setItem(row, 2, new QTableWidgetItem(host.validation));
        hostsTable->setItem(row, 3, new QTableWidgetItem(host.path));
    }

    if (!error.isEmpty()) {
        details->setPlainText("Host list failed\n" + error);
        return;
    }

    if (hosts.isEmpty()) {
        details->setPlainText("No hosts found");
        return;
    }

    hostsTable->selectRow(0);
    details->setPlainText(adapter->inspectHost(hosts.at(0).path));
}

void populateIdentityTable(QTableWidget *identitiesTable, QPlainTextEdit *details,
                           const CliAdapter *adapter) {
    QString error;
    const QList<IdentitySummary> identities = adapter->listIdentities(&error);

    identitiesTable->setRowCount(identities.size());
    for (int row = 0; row < identities.size(); ++row) {
        const IdentitySummary &identity = identities.at(row);
        auto *name = new QTableWidgetItem(identity.displayName);
        name->setData(Qt::UserRole, identity.path);
        identitiesTable->setItem(row, 0, name);
        identitiesTable->setItem(row, 1, new QTableWidgetItem(identity.id));
        identitiesTable->setItem(row, 2, new QTableWidgetItem(identity.validation));
        identitiesTable->setItem(row, 3, new QTableWidgetItem(identity.path));
    }

    if (!error.isEmpty()) {
        details->setPlainText("Identity list failed\n" + error);
        return;
    }

    if (identities.isEmpty()) {
        details->setPlainText("No identities found");
        return;
    }

    identitiesTable->selectRow(0);
    details->setPlainText(adapter->inspectIdentity(identities.at(0).path));
}

} // namespace

QWidget *explorePage(const WorkspaceConfig &config) {
    auto *page = new QWidget;
    auto *layout = new QVBoxLayout(page);
    layout->setContentsMargins(16, 12, 16, 12);
    layout->setSpacing(10);

    layout->addWidget(sectionLabel("Explore"));
    layout->addWidget(separator());
    layout->addWidget(table({"Resource", "Protocol", "State"},
                            {{"Local start", "gemini", "available"},
                             {"Saved capsule", "gemini", "empty"},
                             {"Offline notes", "file", "available"}}));
    layout->addWidget(sectionLabel("Active Roots"));
    layout->addWidget(table({"Root", "Status", "Path"},
                            {{"Config source", config.configSource, config.configPath},
                             {"Repository", rootStatus(config.repoRoot), config.repoRoot},
                             {"Projects", rootStatus(config.projectsRoot),
                              config.projectsRoot},
                             {"Hosts", rootStatus(config.hostsRoot), config.hostsRoot},
                             {"Identities", rootStatus(config.identitiesRoot),
                              config.identitiesRoot},
                             {"Audio metadata", rootStatus(config.audioMetadataRoot),
                              config.audioMetadataRoot}}));

    layout->addWidget(sectionLabel("User Settings"));
    auto *settings = new QWidget(page);
    auto *form = new QFormLayout(settings);
    form->setContentsMargins(0, 0, 0, 0);
    form->setSpacing(8);

    auto *projectsRoot = new QLineEdit(config.projectsRoot, settings);
    auto *hostsRoot = new QLineEdit(config.hostsRoot, settings);
    auto *identitiesRoot = new QLineEdit(config.identitiesRoot, settings);
    auto *audioMetadataRoot = new QLineEdit(config.audioMetadataRoot, settings);
    form->addRow("Projects", rootEditorRow(settings, "Projects Root", projectsRoot));
    form->addRow("Hosts", rootEditorRow(settings, "Hosts Root", hostsRoot));
    form->addRow("Identities",
                 rootEditorRow(settings, "Identities Root", identitiesRoot));
    form->addRow("Audio metadata",
                 rootEditorRow(settings, "Audio Metadata Root", audioMetadataRoot));

    auto *actions = new QWidget(settings);
    auto *actionsLayout = new QHBoxLayout(actions);
    actionsLayout->setContentsMargins(0, 0, 0, 0);
    auto *save = new QPushButton("Save User Settings", actions);
    auto *status = new QLabel("User config: " + WorkspaceConfig::userConfigPath(), actions);
    actionsLayout->addWidget(save);
    actionsLayout->addWidget(status, 1);
    form->addRow("", actions);

    QObject::connect(save, &QPushButton::clicked, [=]() {
        WorkspaceConfig edited = config;
        edited.projectsRoot = projectsRoot->text();
        edited.hostsRoot = hostsRoot->text();
        edited.identitiesRoot = identitiesRoot->text();
        edited.audioMetadataRoot = audioMetadataRoot->text();

        QString error;
        if (!WorkspaceConfig::saveUserConfig(edited, &error)) {
            status->setText(error);
            return;
        }

        status->setText("Saved user settings. Changes apply on next launch.");
    });

    layout->addWidget(settings);
    layout->addStretch();
    return page;
}

QWidget *createPage(const CliAdapter *adapter) {
    auto *page = new QWidget;
    auto *layout = new QVBoxLayout(page);
    layout->setContentsMargins(16, 12, 16, 12);
    layout->setSpacing(10);

    layout->addWidget(sectionLabel("Create"));
    layout->addWidget(separator());

    auto *refresh = new QPushButton("Refresh");
    auto *toolbar = new QWidget;
    auto *toolbarLayout = new QHBoxLayout(toolbar);
    toolbarLayout->setContentsMargins(0, 0, 0, 0);
    toolbarLayout->addWidget(refresh);
    toolbarLayout->addStretch();
    layout->addWidget(toolbar);

    auto *newProject = new QWidget(page);
    auto *newProjectForm = new QFormLayout(newProject);
    newProjectForm->setContentsMargins(0, 0, 0, 0);
    newProjectForm->setSpacing(8);

    auto *newProjectId = new QLineEdit(newProject);
    auto *newProjectName = new QLineEdit(newProject);
    auto *newProjectType = new QComboBox(newProject);
    newProjectType->addItems({"capsule",
                              "gemlog",
                              "gopherhole",
                              "spartan-site",
                              "audio-series",
                              "feed",
                              "pubnix-home",
                              "documentation-archive",
                              "classroom-assignment",
                              "mixed-publication"});

    auto *createActions = new QWidget(newProject);
    auto *createActionsLayout = new QHBoxLayout(createActions);
    createActionsLayout->setContentsMargins(0, 0, 0, 0);
    auto *createProject = new QPushButton("Create", createActions);
    auto *createStatus = new QLabel("Create: idle", createActions);
    createStatus->setWordWrap(true);
    createActionsLayout->addWidget(createProject);
    createActionsLayout->addWidget(createStatus, 1);

    newProjectForm->addRow("ID", newProjectId);
    newProjectForm->addRow("Name", newProjectName);
    newProjectForm->addRow("Type", newProjectType);
    newProjectForm->addRow("", createActions);
    layout->addWidget(newProject);

    auto *splitter = new QSplitter(Qt::Vertical);
    auto currentDocument = std::make_shared<ProjectDocument>();

    auto *projectArea = new QWidget;
    auto *projectLayout = new QVBoxLayout(projectArea);
    projectLayout->setContentsMargins(0, 0, 0, 0);
    projectLayout->addWidget(new QLabel("Projects"));
    auto *projectsTable = table({"Project", "Type", "Validation", "Path"}, {});
    projectLayout->addWidget(projectsTable);
    auto *projectDetails = new QPlainTextEdit;
    projectDetails->setReadOnly(true);
    projectLayout->addWidget(projectDetails);
    splitter->addWidget(projectArea);

    auto *authorArea = new QWidget;
    auto *authorLayout = new QVBoxLayout(authorArea);
    authorLayout->setContentsMargins(0, 0, 0, 0);

    auto *authorToolbar = new QWidget(authorArea);
    auto *authorToolbarLayout = new QHBoxLayout(authorToolbar);
    authorToolbarLayout->setContentsMargins(0, 0, 0, 0);
    auto *contentPath = new QLabel("No project selected", authorToolbar);
    contentPath->setWordWrap(true);
    auto *reloadContent = new QPushButton("Reload", authorToolbar);
    auto *saveContent = new QPushButton("Save", authorToolbar);
    saveContent->setEnabled(false);
    reloadContent->setEnabled(false);
    authorToolbarLayout->addWidget(new QLabel("Project Content", authorToolbar));
    authorToolbarLayout->addWidget(contentPath, 1);
    authorToolbarLayout->addWidget(reloadContent);
    authorToolbarLayout->addWidget(saveContent);
    authorLayout->addWidget(authorToolbar);

    auto *contentSplitter = new QSplitter(Qt::Horizontal, authorArea);
    auto *editor = new QPlainTextEdit(contentSplitter);
    editor->setPlaceholderText("Select a project with a content index");
    auto *preview = new QTextBrowser(contentSplitter);
    preview->setOpenExternalLinks(false);
    preview->setHtml(renderGemtextPreview(""));
    contentSplitter->addWidget(editor);
    contentSplitter->addWidget(preview);
    contentSplitter->setStretchFactor(0, 1);
    contentSplitter->setStretchFactor(1, 1);
    authorLayout->addWidget(contentSplitter, 1);

    auto *contentStatus = new QLabel("Content: idle", authorArea);
    contentStatus->setWordWrap(true);
    authorLayout->addWidget(contentStatus);
    splitter->addWidget(authorArea);

    auto *recordingArea = new QWidget;
    auto *recordingLayout = new QVBoxLayout(recordingArea);
    recordingLayout->setContentsMargins(0, 0, 0, 0);
    recordingLayout->addWidget(new QLabel("Recordings"));
    auto *recordingsTable =
        table({"Recording", "ID", "Validation", "Playable", "Path"}, {});
    recordingLayout->addWidget(recordingsTable);
    auto *recordingDetails = new QPlainTextEdit;
    recordingDetails->setReadOnly(true);
    recordingLayout->addWidget(recordingDetails);
    splitter->addWidget(recordingArea);
    splitter->setStretchFactor(0, 1);
    splitter->setStretchFactor(1, 2);
    splitter->setStretchFactor(2, 1);
    layout->addWidget(splitter, 1);

    auto loadProjectContent = [=](const QString &path) {
        *currentDocument = adapter->loadProjectDocument(path);
        if (!currentDocument->ok) {
            contentPath->setText("No content loaded");
            contentStatus->setText(currentDocument->error);
            editor->clear();
            preview->setHtml(renderGemtextPreview(""));
            saveContent->setEnabled(false);
            reloadContent->setEnabled(false);
            return;
        }

        contentPath->setText(currentDocument->contentPath);
        contentStatus->setText("Loaded: " + currentDocument->title);
        editor->setPlainText(currentDocument->text);
        preview->setHtml(renderGemtextPreview(currentDocument->text));
        saveContent->setEnabled(true);
        reloadContent->setEnabled(true);
    };

    QObject::connect(createProject, &QPushButton::clicked, [=]() {
        const QString id = newProjectId->text().trimmed();
        const QString name = newProjectName->text().trimmed();
        const QString projectType = newProjectType->currentText();
        if (id.isEmpty() || name.isEmpty()) {
            createStatus->setText("ID and name are required");
            return;
        }

        const ProjectCreateResult created =
            adapter->createProject(id, name, projectType);
        if (!created.ok) {
            createStatus->setText(created.error);
            return;
        }

        createStatus->setText("Created: " + created.projectPath);
        newProjectId->clear();
        newProjectName->clear();
        populateProjectTable(projectsTable, projectDetails, adapter);
        if (selectProjectByPath(projectsTable, created.projectPath)) {
            projectDetails->setPlainText(adapter->inspectProject(created.projectPath));
            loadProjectContent(created.projectPath);
        }
    });

    QObject::connect(refresh, &QPushButton::clicked, [=]() {
        populateProjectTable(projectsTable, projectDetails, adapter);
        populateRecordingTable(recordingsTable, recordingDetails, adapter);
    });

    QObject::connect(projectsTable, &QTableWidget::currentCellChanged,
                     [=](int row, int, int, int) {
                         if (row < 0) {
                             return;
                         }
                         auto *item = projectsTable->item(row, 0);
                         if (item == nullptr) {
                             return;
                         }
                         const QString path = item->data(Qt::UserRole).toString();
                         projectDetails->setPlainText(adapter->inspectProject(path));
                         loadProjectContent(path);
                     });

    QObject::connect(editor, &QPlainTextEdit::textChanged, [=]() {
        const QString text = editor->toPlainText();
        preview->setHtml(renderGemtextPreview(text));
        if (!currentDocument->ok) {
            return;
        }

        if (text == currentDocument->text) {
            contentStatus->setText("Loaded: " + currentDocument->title);
        } else {
            contentStatus->setText("Edited: " + currentDocument->contentPath);
        }
    });

    QObject::connect(reloadContent, &QPushButton::clicked, [=]() {
        if (!currentDocument->projectPath.isEmpty()) {
            loadProjectContent(currentDocument->projectPath);
        }
    });

    QObject::connect(saveContent, &QPushButton::clicked, [=]() {
        QString error;
        if (!adapter->saveProjectDocument(*currentDocument, editor->toPlainText(),
                                          &error)) {
            contentStatus->setText(error);
            return;
        }

        currentDocument->text = editor->toPlainText();
        const QString validation =
            adapter->projectValidationState(currentDocument->projectPath);
        contentStatus->setText("Saved: " + currentDocument->contentPath +
                               " (" + validation + ")");
        projectDetails->setPlainText(
            adapter->inspectProject(currentDocument->projectPath) +
            "\n\nValidation: " + validation);
    });

    QObject::connect(recordingsTable, &QTableWidget::currentCellChanged,
                     [=](int row, int, int, int) {
                         if (row < 0) {
                             return;
                         }
                         auto *item = recordingsTable->item(row, 0);
                         if (item == nullptr) {
                             return;
                         }
                         const QString path = item->data(Qt::UserRole).toString();
                         recordingDetails->setPlainText(adapter->inspectRecording(path));
                     });

    populateProjectTable(projectsTable, projectDetails, adapter);
    populateRecordingTable(recordingsTable, recordingDetails, adapter);
    return page;
}

QWidget *publishPage(const CliAdapter *adapter) {
    auto *page = new QWidget;
    auto *layout = new QVBoxLayout(page);
    layout->setContentsMargins(16, 12, 16, 12);
    layout->setSpacing(10);

    layout->addWidget(sectionLabel("Publish"));
    layout->addWidget(separator());

    auto *toolbar = new QWidget;
    auto *toolbarLayout = new QHBoxLayout(toolbar);
    toolbarLayout->setContentsMargins(0, 0, 0, 0);
    auto *target = new QLineEdit;
    target->setPlaceholderText("target");
    auto *preview = new QPushButton("Preview");
    auto *refresh = new QPushButton("Refresh");
    toolbarLayout->addWidget(new QLabel("Target"));
    toolbarLayout->addWidget(target);
    toolbarLayout->addWidget(preview);
    toolbarLayout->addWidget(refresh);
    toolbarLayout->addStretch();
    layout->addWidget(toolbar);

    auto *projectsTable = table({"Project", "Target", "Path"}, {});
    layout->addWidget(projectsTable);
    auto *plan = new QPlainTextEdit;
    plan->setReadOnly(true);
    layout->addWidget(plan);

    auto runPreview = [=]() {
        const int row = projectsTable->currentRow();
        if (row < 0) {
            plan->setPlainText("No project selected");
            return;
        }
        auto *item = projectsTable->item(row, 0);
        if (item == nullptr) {
            plan->setPlainText("No project selected");
            return;
        }
        const QString path = item->data(Qt::UserRole).toString();
        plan->setPlainText(
            renderPublishPreview(adapter->previewPublication(path, target->text())));
    };

    QObject::connect(refresh, &QPushButton::clicked, [=]() {
        populatePublishTable(projectsTable, target, plan, adapter);
    });

    QObject::connect(preview, &QPushButton::clicked, runPreview);

    QObject::connect(projectsTable, &QTableWidget::currentCellChanged,
                     [=](int row, int, int, int) {
                         if (row < 0) {
                             return;
                         }
                         auto *item = projectsTable->item(row, 0);
                         if (item == nullptr) {
                             return;
                         }
                         target->setText(item->data(Qt::UserRole + 1).toString());
                         runPreview();
                     });

    populatePublishTable(projectsTable, target, plan, adapter);
    return page;
}

QWidget *operatePage(const CliAdapter *adapter) {
    auto *page = new QWidget;
    auto *layout = new QVBoxLayout(page);
    layout->setContentsMargins(16, 12, 16, 12);
    layout->setSpacing(10);

    layout->addWidget(sectionLabel("Operate"));
    layout->addWidget(separator());

    auto *toolbar = new QWidget;
    auto *toolbarLayout = new QHBoxLayout(toolbar);
    toolbarLayout->setContentsMargins(0, 0, 0, 0);
    auto *refresh = new QPushButton("Refresh");
    toolbarLayout->addWidget(refresh);
    toolbarLayout->addStretch();
    layout->addWidget(toolbar);

    auto *splitter = new QSplitter(Qt::Vertical);

    auto *tables = new QWidget;
    auto *tablesLayout = new QHBoxLayout(tables);
    tablesLayout->setContentsMargins(0, 0, 0, 0);
    tablesLayout->setSpacing(10);

    auto *hostArea = new QWidget;
    auto *hostLayout = new QVBoxLayout(hostArea);
    hostLayout->setContentsMargins(0, 0, 0, 0);
    hostLayout->addWidget(new QLabel("Hosts"));
    auto *hostsTable = table({"Host", "Address", "Validation", "Path"}, {});
    hostLayout->addWidget(hostsTable);
    tablesLayout->addWidget(hostArea);

    auto *identityArea = new QWidget;
    auto *identityLayout = new QVBoxLayout(identityArea);
    identityLayout->setContentsMargins(0, 0, 0, 0);
    identityLayout->addWidget(new QLabel("Identities"));
    auto *identitiesTable = table({"Identity", "ID", "Validation", "Path"}, {});
    identityLayout->addWidget(identitiesTable);
    tablesLayout->addWidget(identityArea);

    splitter->addWidget(tables);

    auto *details = new QPlainTextEdit;
    details->setReadOnly(true);
    splitter->addWidget(details);
    splitter->setStretchFactor(0, 2);
    splitter->setStretchFactor(1, 1);
    layout->addWidget(splitter, 1);

    QObject::connect(refresh, &QPushButton::clicked, [=]() {
        populateHostTable(hostsTable, details, adapter);
        populateIdentityTable(identitiesTable, details, adapter);
    });

    QObject::connect(hostsTable, &QTableWidget::currentCellChanged,
                     [=](int row, int, int, int) {
                         if (row < 0) {
                             return;
                         }
                         auto *item = hostsTable->item(row, 0);
                         if (item == nullptr) {
                             return;
                         }
                         const QString path = item->data(Qt::UserRole).toString();
                         details->setPlainText(adapter->inspectHost(path));
                     });

    QObject::connect(identitiesTable, &QTableWidget::currentCellChanged,
                     [=](int row, int, int, int) {
                         if (row < 0) {
                             return;
                         }
                         auto *item = identitiesTable->item(row, 0);
                         if (item == nullptr) {
                             return;
                         }
                         const QString path = item->data(Qt::UserRole).toString();
                         details->setPlainText(adapter->inspectIdentity(path));
                     });

    populateHostTable(hostsTable, details, adapter);
    populateIdentityTable(identitiesTable, details, adapter);
    return page;
}
