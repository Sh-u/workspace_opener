## workspace_opener

Workspace Opener is an easy to use TUI application for creating and running Windows Terminal configs.

## Download

Visit the [Releases page](https://github.com/Sh-u/workspace_opener/releases) and navigate to the "Assets" dropdown under the latest release, then download the zip file titled `workspace_opener.zip` and choose the latest version.
**DO NOT** download by clicking the green "Code"/"Download" file on the home page of this repository as that will only download the source code, which isn't what you want.

## Installation

- **Prerequisites**
  * Windows 10 or Windows 11
  * Windows Terminal (wt.exe) from the Microsoft App Store
  * Command Prompt or Windows Powershell

- Extract the zip archive.

- Run `workspace_opener.exe` **as an administrator** inside the folder.

## Guide

- Create your first preset by following the `Create Preset` prompts.
  * Name (profile name)
  * Tabs (number of tabs to open)
  e.g. 3 tabs
  ![wt_tabs](assets/wt_tabs.png)
  * Windows (number of windows/pane)
  e.g 4 windows
  ![wt_windows](assets/wt_windows.png)
  * Args (which commands to run upon opening)
  e.g args for windows #1 `cd Rust, ls` (separated by commas)

- Edit your preset profile/shell
  * Select `Choose Preset` option then press `E` on target preset.
  * Wt profile (name of windows terminal profile you want to run. Leave empty if none.)
  * Init shell (a shell from which the commands will be run. Recommended powershell.)
  * Target shell (a shell that will be opened in which window. It's the actual shell that you want to work with)

- Run the config by pressing `Enter` on selected preset name
  
- Warnings
  * Application may not work correctly if you do not open as an admin.
  * Do not change the config in any other way than through the app directly.


