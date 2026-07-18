#pragma once

#include <QString>
#include <QStringList>

struct WorkspaceConfig {
    QString repoRoot;
    QString configPath;
    QString configSource;
    QString projectsRoot;
    QString hostsRoot;
    QString identitiesRoot;
    QString audioMetadataRoot;

    QStringList missingRootMessages() const;

    static WorkspaceConfig defaults(const QString &repoRoot);
    static WorkspaceConfig load(const QString &repoRoot, const QString &explicitConfigPath,
                                bool allowUserConfig, QString *warning);
    static bool saveUserConfig(const WorkspaceConfig &config, QString *error);
    static QString userConfigPath();
};
