# Requirements Document

## Introduction

Auto-update system for Academix, a Tauri 2 desktop application targeting Windows, macOS, and Linux. The system detects available updates from GitHub Releases, verifies platform and architecture compatibility, and provides a seamless one-click update experience with automatic application restart. It leverages `tauri-plugin-updater` for secure, signed update delivery.

## Glossary

- **Updater**: The Tauri updater plugin subsystem (`tauri-plugin-updater`) responsible for checking, downloading, verifying, and installing application updates
- **Update_Endpoint**: The GitHub Releases URL configured as the update source, returning a static JSON manifest with platform-specific artifacts
- **Update_Manifest**: The JSON response from the Update_Endpoint containing version, platform URLs, signatures, and release notes
- **Platform_Target**: A combination of operating system and CPU architecture (e.g., `windows-x86_64`, `darwin-aarch64`, `linux-x86_64`)
- **Update_Notification**: The UI element displayed to the user when a new version is available, containing version information, release notes, and a download button
- **Signature_Verification**: The cryptographic validation of update artifacts using a public key embedded in the application configuration
- **Install_Mode**: The Windows-specific installation behavior (`passive` shows a progress bar, `quiet` installs silently)

## Requirements

### Requirement 1: Update Detection

**User Story:** As an administrator, I want the application to automatically check for updates, so that I am always aware when a new version is available.

#### Acceptance Criteria

1. WHEN the main window becomes ready, THE Updater SHALL check the Update_Endpoint for a newer version within 10 seconds of the window ready event
2. WHILE the application is running, THE Updater SHALL periodically check the Update_Endpoint for updates at a configurable interval between 1 and 24 hours (default: 4 hours), persisting the configured value across application restarts
3. WHEN the Update_Endpoint returns a version string that is semantically higher than the current application version according to SemVer comparison, THE Updater SHALL notify the frontend with the available version number and a flag indicating whether the update is mandatory or optional
4. IF the Update_Endpoint is unreachable or returns a non-success HTTP status, THEN THE Updater SHALL log the failure reason and retry at the next scheduled interval without displaying an error to the user or disrupting application functionality
5. IF the Update_Endpoint returns a response that cannot be parsed as a valid version payload, THEN THE Updater SHALL treat the check as failed, log the parsing error, and retry at the next scheduled interval without notifying the user
6. WHEN the Updater successfully determines no update is available, THE Updater SHALL record the timestamp of the last successful check and not notify the frontend

### Requirement 2: Platform Compatibility Verification

**User Story:** As an administrator, I want the system to verify that updates are compatible with my platform, so that I never receive an update that cannot be installed on my machine.

#### Acceptance Criteria

1. WHEN the Updater receives an Update_Manifest, THE Updater SHALL match the current Platform_Target against the `platforms` keys in the manifest using exact string comparison of the constructed platform key
2. IF the Update_Manifest does not contain an artifact entry whose platform key matches the current Platform_Target, THEN THE Updater SHALL discard the update, not notify the user, and not store the manifest for later retry
3. THE Updater SHALL construct the platform key by concatenating the Tauri-provided `{{target}}` and `{{arch}}` identifiers in the format `{{target}}-{{arch}}` and SHALL use this key for all platform matching operations
4. IF the Updater cannot determine the current platform target or architecture at runtime, THEN THE Updater SHALL log an error indicating the missing platform identifier and SHALL not proceed with the update check
5. WHEN the Update_Manifest contains an artifact for the current Platform_Target, THE Updater SHALL verify that the artifact entry includes a non-empty download URL of 2048 characters or fewer, and SHALL consider the platform match valid only when the URL passes this validation constraint

### Requirement 3: Update Notification

**User Story:** As an administrator, I want to see a clear notification when an update is available, so that I can decide when to download and install it.

#### Acceptance Criteria

1. WHEN an update is available for the current Platform_Target, THE Update_Notification SHALL display the new version number formatted as semantic version (MAJOR.MINOR.PATCH) and release notes with a maximum length of 5000 characters
2. WHEN an update is available, THE Update_Notification SHALL display a "Download" button that initiates the update download process when activated
3. WHILE no update is available, THE Update_Notification SHALL not be rendered in the user interface
4. WHEN the user dismisses the Update_Notification, THE Updater SHALL not show the notification again until the next scheduled check detects a version different from the previously dismissed version
5. IF the release notes cannot be retrieved from the update server, THEN THE Update_Notification SHALL display the new version number and a message indicating that release notes are unavailable
6. WHEN the Update_Notification is displayed, THE Update_Notification SHALL remain visible until the user either activates the download button or explicitly dismisses it

### Requirement 4: Download and Installation

**User Story:** As an administrator, I want to download and install updates with one click, so that the update process is quick and requires minimal effort.

#### Acceptance Criteria

1. WHEN the user clicks the download button, THE Updater SHALL download the update artifact from the URL specified in the Update_Manifest and disable the download button to prevent duplicate requests
2. WHILE the download is in progress, THE Update_Notification SHALL display a progress indicator showing download percentage updated at least every 2 seconds
3. IF the download fails due to network error or server unavailability, THEN THE Updater SHALL abort the download, display an error message indicating the failure reason, and re-enable the download button to allow retry
4. IF the download does not complete within 300 seconds, THEN THE Updater SHALL abort the download, display a timeout error message, and re-enable the download button to allow retry
5. IF the download fails due to a reason not covered by AC3 or AC4, THEN THE Updater SHALL display an error message describing the failure and re-enable the download button to allow retry
6. WHEN the download completes, THE Updater SHALL verify the artifact using Signature_Verification before proceeding with installation
7. IF Signature_Verification fails, THEN THE Updater SHALL abort the installation, delete the downloaded artifact, display an error message indicating signature verification failed, and log the failure with timestamp and artifact URL
8. WHEN Signature_Verification succeeds, THE Updater SHALL install the update using the platform-appropriate installer and prevent the user from initiating another download or install until the current installation completes
9. WHERE the platform is Windows, THE Updater SHALL use passive Install_Mode to show installation progress
10. IF passive Install_Mode is unavailable for the specific update package, THEN THE Updater SHALL fall back to interactive installation mode
11. IF installation fails, THEN THE Updater SHALL display an error message indicating the installation failure reason, preserve the current application version unchanged, and log the failure with timestamp and error details

### Requirement 5: Application Restart

**User Story:** As an administrator, I want the application to restart automatically after an update is installed, so that I can immediately use the new version without manual intervention.

#### Acceptance Criteria

1. WHEN the update installation completes successfully, THE Updater SHALL automatically restart the application using the Tauri process relaunch mechanism within 3 seconds of installation completion
2. WHEN the application restarts after an update, THE Updater SHALL launch the newly installed version and display the updated version number in the application interface
3. IF the restart fails within 5 seconds of the relaunch attempt, THEN THE Updater SHALL display a message instructing the user to manually close and reopen the application, including the installed version identifier
4. WHILE the application is restarting after an update, THE Updater SHALL preserve the user's authenticated session so that re-login is not required after the restart

### Requirement 6: Update Signing Configuration

**User Story:** As a developer, I want updates to be cryptographically signed, so that users are protected from tampered or malicious update artifacts.

#### Acceptance Criteria

1. THE Updater SHALL require a valid Ed25519 public key configured in `tauri.conf.json` under `plugins.updater.pubkey`, and SHALL refuse to check for updates if the key is absent or malformed
2. WHEN a downloaded artifact's cryptographic signature does not match the public key and the corresponding `.sig` file from the Update_Manifest, THEN THE Updater SHALL reject the artifact, discard the downloaded bytes, and present an error message indicating signature verification failure without installing or applying the artifact
3. IF a signing key pair does not exist at the path referenced by the build environment, THEN the build system SHALL generate one using `tauri signer generate` during initial project setup and store the private key in a location accessible only to the CI service account
4. THE build pipeline SHALL use the `TAURI_SIGNING_PRIVATE_KEY` environment variable to sign update artifacts during the release build process, and SHALL fail the build with a non-zero exit code if the variable is unset or empty
5. IF the `TAURI_SIGNING_PRIVATE_KEY` is protected by a passphrase, THEN the build pipeline SHALL read the passphrase from the `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` environment variable during active builds and SHALL fail the build with a non-zero exit code if the passphrase is unset or incorrect at build time

### Requirement 7: Update Endpoint Configuration

**User Story:** As a developer, I want the update endpoint to point to GitHub Releases, so that the existing release workflow serves as the update distribution channel.

#### Acceptance Criteria

1. THE Updater SHALL configure the Update_Endpoint as a GitHub Releases URL with platform variable substitution (`{{target}}` and `{{arch}}`), forming a URL pattern of `https://github.com/{owner}/{repo}/releases/latest/download/{app_name}-{{target}}-{{arch}}.json`
2. THE Updater SHALL support TLS-only connections (HTTPS) to the Update_Endpoint in production builds and SHALL reject any HTTP (non-TLS) endpoint configuration
3. WHEN the Update_Endpoint returns HTTP 204, THE Updater SHALL interpret it as no update available and SHALL retain the current installed version without further action
4. WHEN the Update_Endpoint returns HTTP 200 with a valid Update_Manifest, THE Updater SHALL proceed with version comparison and platform matching against the currently installed version
5. IF the Update_Endpoint connection fails or returns an HTTP status code other than 200 or 204, THEN THE Updater SHALL treat it as a transient network error, retry the request up to 3 times with exponential backoff starting at 1 second, and if all retries fail, notify the user that the update check could not be completed
6. IF the Update_Endpoint returns HTTP 200 but the response body is not a valid Update_Manifest (malformed JSON or missing required fields), THEN THE Updater SHALL discard the response, log the parsing failure, and notify the user that the update check encountered an invalid response
7. WHEN the Updater resolves `{{target}}` and `{{arch}}` variables, THE Updater SHALL substitute them with the current operating system identifier and CPU architecture identifier matching the Tauri updater platform naming convention
