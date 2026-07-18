#include "cli_adapter.h"
#include "workspace_config.h"
#include "workspace_pages.h"

#include <QApplication>
#include <QButtonGroup>
#include <QDir>
#include <QHBoxLayout>
#include <QListWidget>
#include <QMainWindow>
#include <QMenu>
#include <QMenuBar>
#include <QPushButton>
#include <QSplitter>
#include <QStackedWidget>
#include <QStatusBar>
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

QString optionValue(const QApplication &app, const QString &name) {
    const QStringList args = app.arguments();
    for (int index = 1; index + 1 < args.size(); ++index) {
        if (args.at(index) == name) {
            return args.at(index + 1);
        }
    }
    return {};
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

    const QString validation = adapter.projectValidationState(created.projectPath);
    if (validation != "valid") {
        err << "workspace project smoke: validation returned " << validation
            << Qt::endl;
        return 1;
    }

    out << "workspace project smoke: created, saved, and validated "
        << created.projectPath << Qt::endl;
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
