# luau-format-gui

Graphical User Interface for [luau-format](https://github.com/TechHog8984/luau-format)

## Obtaining

Either download the latest of the [github releases](https://github.com/TechHog8984/luau-format-gui/releases) or compile yourself with cargo.

## Usage

Simply run the executable.
<br>
It will first attempt to find `luau-format` in PATH.
<br>
If it doesn't exist, it will then attempt to download the applicable binary for your system from the latest github release. The UI will not open until this process is complete.
<br>
If the above request fails (or if any other error occurs while locating/fetching the binary), an error will appear in stdout and the program will crash. This means that if you didn't run the application via the command line, you may be quite confused. Sorry!
<br>
Press `Open file...` to open a file dialog and choose input location. Upon valid selection, luau-format will try to format the code. If it succeeds, you will see the formatted output in the code editor below. Otherwise, you will see an error below the code editor.
<br>
Press `Save to file...` to open a file dialog and choose output location. Whatever is inside of the code editor will get saved to the given location. Any errors encountered will appear below the code editor.
<br>
Press `Reset editor...` to revert the editor contents back to the original formatted output.
<br>
When modifying any of the available options, luau-format will be rerun, <b>ignoring any changes made in the code editor!</b>

## Contributing

This program is in a pretty bad state (bad code & start logic), so I am warmly welcoming contributions! Just go ahead and make an issue or pull request.

## TODO

* all options
