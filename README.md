## workspace_opener

Workspace Opener is an easy to use TUI application for creating and running Windows Terminal configs.

## Download

Visit the [Releases page](https://github.com/Sh-u/workspace_opener/releases) and navigate to the "Assets" dropdown under the latest release, then download the zip file titled `workspace_opener.rar` and choose the latest version.
**DO NOT** download by clicking the green "Code"/"Download" file on the home page of this repository, as that will only download the source code, which isn't what you want.

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
  e.g args for windows #1 ``ls, cd Rust` Projects, pwd`` (separated by commas)

- Edit your preset profile by selecting `Choose Preset` option then pressing `E` on target preset.
  * Wt profile (name of windows terminal profile you want to run. Leave it empty if none.)
  * Init shell (a shell from which the commands will be run. Recommended powershell.)
  * Target shell (a shell that will be opened in which window. It's the actual shell that you want to work with)

- Run the config by pressing `Enter` on selected preset name
  
## Warnings
  * The Application may not work correctly if you do not open it as an admin.
  * Do not change the config in any other way than through the app directly.

## Common Issues
  * Password prompts in WSL
    - The dirty way is to pipe: ``echo \"password\" | sudo -S <command>``
    - Cleaner way is to run wsl: 
      - enter `sudo visudo`
      - add *NOPASSWD* for specified services like `%sudo ALL=(ALL) NOPASSWD: /usr/sbin/service redis-server start,/usr/sbin/service postgresql start`
  * Command not found
    - Make sure the path is correct by running `echo $PATH` 
    - Provide an explicit path: instead of `npm run` pass `/home/username/bin/npm run` etc.
    - If above does not fix, make sure to install it in `/usr/bin` 


