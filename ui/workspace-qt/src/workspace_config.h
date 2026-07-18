#pragma once

#include <QString>

struct WorkspaceConfig {
    QString repoRoot;
    QString projectsRoot;
    QString hostsRoot;
    QString identitiesRoot;
    QString audioMetadataRoot;

    static WorkspaceConfig defaults(const QString &repoRoot);
    static WorkspaceConfig load(const QString &repoRoot, const QString &configPath,
                                QString *warning);
};
