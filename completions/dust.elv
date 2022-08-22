
use builtin;
use str;

set edit:completion:arg-completer[dust] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'dust'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'dust'= {
            cand -d 'Depth to show'
            cand --depth 'Depth to show'
            cand -n 'Number of lines of output to show. (Default is terminal_height - 10)'
            cand --number-of-lines 'Number of lines of output to show. (Default is terminal_height - 10)'
            cand -X 'Exclude any file or directory with this name'
            cand --ignore-directory 'Exclude any file or directory with this name'
            cand -v 'Exclude filepaths matching this regex. To ignore png files type: -v "\.png$" '
            cand --invert-filter 'Exclude filepaths matching this regex. To ignore png files type: -v "\.png$" '
            cand -e 'Only include filepaths matching this regex. For png files type: -e "\.png$" '
            cand --filter 'Only include filepaths matching this regex. For png files type: -e "\.png$" '
            cand -w 'Specify width of output overriding the auto detection of terminal width'
            cand --terminal_width 'Specify width of output overriding the auto detection of terminal width'
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand -V 'Print version information'
            cand --version 'Print version information'
            cand -p 'Subdirectories will not have their path shortened'
            cand --full-paths 'Subdirectories will not have their path shortened'
            cand -x 'Only count the files and directories on the same filesystem as the supplied directory'
            cand --limit-filesystem 'Only count the files and directories on the same filesystem as the supplied directory'
            cand -s 'Use file length instead of blocks'
            cand --apparent-size 'Use file length instead of blocks'
            cand -r 'Print tree upside down (biggest highest)'
            cand --reverse 'Print tree upside down (biggest highest)'
            cand -c 'No colors will be printed (Useful for commands like: watch)'
            cand --no-colors 'No colors will be printed (Useful for commands like: watch)'
            cand -b 'No percent bars or percentages will be displayed'
            cand --no-percent-bars 'No percent bars or percentages will be displayed'
            cand --skip-total 'No total row will be displayed'
            cand -f 'Directory ''size'' is number of child files/dirs not disk size'
            cand --filecount 'Directory ''size'' is number of child files/dirs not disk size'
            cand -i 'Do not display hidden files'
            cand --ignore_hidden 'Do not display hidden files'
            cand -t 'show only these file types'
            cand --file_types 'show only these file types'
            cand -H 'print sizes in powers of 1000 (e.g., 1.1G)'
            cand --si 'print sizes in powers of 1000 (e.g., 1.1G)'
        }
    ]
    $completions[$command]
}
