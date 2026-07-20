#pragma once

#include <functional>

class CliAdapter;
class QString;
struct WorkspaceConfig;
class QWidget;

using FeedDiagnosticCreateHandoff =
    std::function<bool(const QString &projectPath, const QString &recordingId)>;

QWidget *explorePage(const WorkspaceConfig &config);
QWidget *createPage(const CliAdapter *adapter);
bool focusCreateProject(QWidget *page, const QString &projectPath,
                        const QString &recordingId);
QWidget *publishPage(const CliAdapter *adapter,
                     const FeedDiagnosticCreateHandoff &openInCreate = {});
QWidget *operatePage(const CliAdapter *adapter);
