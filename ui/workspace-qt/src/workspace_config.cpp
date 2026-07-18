#include "workspace_config.h"

#include <QDir>
#include <QFileInfo>
#include <QList>
#include <QPair>
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

QStringList WorkspaceConfig::missingRootMessages() const {
    QStringList messages;
    const QList<QPair<QString, QString>> roots = {
        {"projects", projectsRoot},
        {"hosts", hostsRoot},
        {"identities", identitiesRoot},
        {"audio metadata", audioMetadataRoot},
    };

    for (const auto &root : roots) {
        if (!QFileInfo::exists(root.second)) {
            messages.append("Configured " + root.first + " root not found: " +
                            root.second);
        }
    }

    return messages;
}

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

bool WorkspaceConfig::saveUserConfig(const WorkspaceConfig &config, QString *error) {
    const QString path = userConfigPath();
    if (path.isEmpty()) {
        if (error != nullptr) {
            *error = "User config location is not available";
        }
        return false;
    }

    const QFileInfo file(path);
    if (!QDir().mkpath(file.absolutePath())) {
        if (error != nullptr) {
            *error = "Could not create user config directory: " + file.absolutePath();
        }
        return false;
    }

    QSettings settings(path, QSettings::IniFormat);
    settings.setValue("roots/projects", QDir::cleanPath(config.projectsRoot));
    settings.setValue("roots/hosts", QDir::cleanPath(config.hostsRoot));
    settings.setValue("roots/identities", QDir::cleanPath(config.identitiesRoot));
    settings.setValue("roots/audio_metadata", QDir::cleanPath(config.audioMetadataRoot));
    settings.sync();

    if (settings.status() != QSettings::NoError) {
        if (error != nullptr) {
            *error = "Could not write user config: " + path;
        }
        return false;
    }

    return true;
}

QString WorkspaceConfig::userConfigPath() {
    const QString configDir =
        QStandardPaths::writableLocation(QStandardPaths::AppConfigLocation);
    if (configDir.isEmpty()) {
        return {};
    }
    return QDir(configDir).filePath("workspace.ini");
}
