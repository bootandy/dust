
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'dust' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'dust'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'dust' {
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'Depth to show')
            [CompletionResult]::new('--depth', 'depth', [CompletionResultType]::ParameterName, 'Depth to show')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Number of lines of output to show. (Default is terminal_height - 10)')
            [CompletionResult]::new('--number-of-lines', 'number-of-lines', [CompletionResultType]::ParameterName, 'Number of lines of output to show. (Default is terminal_height - 10)')
            [CompletionResult]::new('-X', 'X', [CompletionResultType]::ParameterName, 'Exclude any file or directory with this name')
            [CompletionResult]::new('--ignore-directory', 'ignore-directory', [CompletionResultType]::ParameterName, 'Exclude any file or directory with this name')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Exclude filepaths matching this regex. To ignore png files type: -v "\.png$" ')
            [CompletionResult]::new('--invert-filter', 'invert-filter', [CompletionResultType]::ParameterName, 'Exclude filepaths matching this regex. To ignore png files type: -v "\.png$" ')
            [CompletionResult]::new('-e', 'e', [CompletionResultType]::ParameterName, 'Only include filepaths matching this regex. For png files type: -e "\.png$" ')
            [CompletionResult]::new('--filter', 'filter', [CompletionResultType]::ParameterName, 'Only include filepaths matching this regex. For png files type: -e "\.png$" ')
            [CompletionResult]::new('-w', 'w', [CompletionResultType]::ParameterName, 'Specify width of output overriding the auto detection of terminal width')
            [CompletionResult]::new('--terminal_width', 'terminal_width', [CompletionResultType]::ParameterName, 'Specify width of output overriding the auto detection of terminal width')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('-p', 'p', [CompletionResultType]::ParameterName, 'Subdirectories will not have their path shortened')
            [CompletionResult]::new('--full-paths', 'full-paths', [CompletionResultType]::ParameterName, 'Subdirectories will not have their path shortened')
            [CompletionResult]::new('-x', 'x', [CompletionResultType]::ParameterName, 'Only count the files and directories on the same filesystem as the supplied directory')
            [CompletionResult]::new('--limit-filesystem', 'limit-filesystem', [CompletionResultType]::ParameterName, 'Only count the files and directories on the same filesystem as the supplied directory')
            [CompletionResult]::new('-s', 's', [CompletionResultType]::ParameterName, 'Use file length instead of blocks')
            [CompletionResult]::new('--apparent-size', 'apparent-size', [CompletionResultType]::ParameterName, 'Use file length instead of blocks')
            [CompletionResult]::new('-r', 'r', [CompletionResultType]::ParameterName, 'Print tree upside down (biggest highest)')
            [CompletionResult]::new('--reverse', 'reverse', [CompletionResultType]::ParameterName, 'Print tree upside down (biggest highest)')
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'No colors will be printed (Useful for commands like: watch)')
            [CompletionResult]::new('--no-colors', 'no-colors', [CompletionResultType]::ParameterName, 'No colors will be printed (Useful for commands like: watch)')
            [CompletionResult]::new('-b', 'b', [CompletionResultType]::ParameterName, 'No percent bars or percentages will be displayed')
            [CompletionResult]::new('--no-percent-bars', 'no-percent-bars', [CompletionResultType]::ParameterName, 'No percent bars or percentages will be displayed')
            [CompletionResult]::new('--skip-total', 'skip-total', [CompletionResultType]::ParameterName, 'No total row will be displayed')
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'Directory ''size'' is number of child files/dirs not disk size')
            [CompletionResult]::new('--filecount', 'filecount', [CompletionResultType]::ParameterName, 'Directory ''size'' is number of child files/dirs not disk size')
            [CompletionResult]::new('-i', 'i', [CompletionResultType]::ParameterName, 'Do not display hidden files')
            [CompletionResult]::new('--ignore_hidden', 'ignore_hidden', [CompletionResultType]::ParameterName, 'Do not display hidden files')
            [CompletionResult]::new('-t', 't', [CompletionResultType]::ParameterName, 'show only these file types')
            [CompletionResult]::new('--file_types', 'file_types', [CompletionResultType]::ParameterName, 'show only these file types')
            [CompletionResult]::new('-H', 'H', [CompletionResultType]::ParameterName, 'print sizes in powers of 1000 (e.g., 1.1G)')
            [CompletionResult]::new('--si', 'si', [CompletionResultType]::ParameterName, 'print sizes in powers of 1000 (e.g., 1.1G)')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
