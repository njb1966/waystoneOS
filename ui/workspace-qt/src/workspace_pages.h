#pragma once

class CliAdapter;
class QWidget;

QWidget *explorePage();
QWidget *createPage(const CliAdapter *adapter);
QWidget *publishPage(const CliAdapter *adapter);
QWidget *operatePage(const CliAdapter *adapter);
