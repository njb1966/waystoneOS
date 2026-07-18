#include "workspace_config.h"

#include <QDir>
#include <QFileInfo>
#include <QSettings>
#include <QStandardPaths>

namespace {

QString absolutePath(const QString &repoRoot, const QString &path) {
    if (path.isEmpty()) {
        return path;
    }

    const QFileInfo info(path);
    if (info.isAbsolute()) {
        return QDir::cleanPath(path);
    }

    return QDir(repoRoot).absoluteFilePath(path);
}

QString configValue(const QSettings &settings, const QString &key,
                    const QString &fallback, const QString &repoRoot) {
    const QString value = settings.value(key, fallback).toString();
    return absolutePath(repoRoot, value);
}

} // namespace

WorkspaceConfig WorkspaceConfig::defaults(const QString &repoRoot) {
    const QString absoluteRepoRoot = QDir(repoRoot).absolutePath();
    WorkspaceConfig config;
    config.repoRoot = absoluteRepoRoot;
    config.configSource = "defaults";
    config.projectsRoot = QDir(absoluteRepoRoot).filePath("examples/projects");
    config.hostsRoot = QDir(absoluteRepoRoot).filePath("examples/connections/hosts");
    config.identitiesRoot =
        QDir(absoluteRepoRoot).filePath("examples/connections/identities");
    config.audioMetadataRoot = QDir(absoluteRepoRoot).filePath(
        "examples/projects/audio-capsule.wayproject/audio/metadata");
    return config;
}

WorkspaceConfig WorkspaceConfig::load(const QString &repoRoot,
                                      const QString &explicitConfigPath,
                                      bool allowUserConfig, QString *warning) {
    WorkspaceConfig config = defaults(repoRoot);
    QString configPath = explicitConfigPath;
    QString source = "explicit";

    if (configPath.isEmpty() && allowUserConfig) {
        const QString userPath = userConfigPath();
        if (QFileInfo::exists(userPath)) {
            configPath = userPath;
            source = "user";
        }
    }

    if (configPath.isEmpty()) {
        return config;
    }

    const QString absoluteConfigPath = absolutePath(config.repoRoot, configPath);
    if (!QFileInfo::exists(absoluteConfigPath)) {
        if (warning != nullptr) {
            *warning = "Workspace config not found; using defaults: " + absoluteConfigPath;
        }
        return config;
    }

    config.configPath = absoluteConfigPath;
    config.configSource = source;
    QSettings settings(absoluteConfigPath, QSettings::IniFormat);
    config.projectsRoot =
        configValue(settings, "roots/projects", config.projectsRoot, config.repoRoot);
    config.hostsRoot = configValue(settings, "roots/hosts", config.hostsRoot, config.repoRoot);
    config.identitiesRoot =
        configValue(settings, "roots/identities", config.identitiesRoot, config.repoRoot);
    config.audioMetadataRoot = configValue(settings, "roots/audio_metadata",
                                           config.audioMetadataRoot, config.repoRoot);
    return config;
}

QString WorkspaceConfig::userConfigPath() {
    const QString configDir =
        QStandardPaths::writableLocation(QStandardPaths::AppConfigLocation);
    if (configDir.isEmpty()) {
        return {};
    }
    return QDir(configDir).filePath("workspace.ini");
}
