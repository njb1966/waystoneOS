#pragma once

class CliAdapter;
struct WorkspaceConfig;
class QWidget;

QWidget *explorePage(const WorkspaceConfig &config);
QWidget *createPage(const CliAdapter *adapter);
QWidget *publishPage(const CliAdapter *adapter);
QWidget *operatePage(const CliAdapter *adapter);
