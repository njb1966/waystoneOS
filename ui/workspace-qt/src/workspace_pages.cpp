#include "workspace_pages.h"

#include "cli_adapter.h"
#include "workspace_config.h"

#include <QAbstractItemView>
#include <QFileInfo>
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
#include <QVBoxLayout>
#include <QWidget>

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

    auto *splitter = new QSplitter(Qt::Vertical);

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
    splitter->setStretchFactor(1, 1);
    layout->addWidget(splitter, 1);

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
