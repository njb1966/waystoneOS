#include "cli_adapter.h"
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
        QTableWidget, QPlainTextEdit {
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
    app.setApplicationName("Waystone Workspace");
    const QString repoRoot = configuredRepoRoot(app);
    const CliAdapter adapter(repoRoot);
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
    pages->addWidget(explorePage());
    pages->addWidget(createPage(&adapter));
    pages->addWidget(publishPage(&adapter));
    pages->addWidget(operatePage(&adapter));
    splitter->addWidget(pages);
    splitter->setStretchFactor(1, 1);
    rootLayout->addWidget(splitter, 1);

    auto setWorkspace = [&](int index) {
        pages->setCurrentIndex(index);
        buttonGroup->button(index)->setChecked(true);
        window.statusBar()->showMessage(
            workspaces.at(index) + "   Audio: Idle   Network: Offline   Project: None");
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
