
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
            cand -T 'Number of threads to use'
            cand --threads 'Number of threads to use'
            cand --config 'Specify a config file to use'
            cand -n 'Number of lines of output to show. (Default is terminal_height - 10)'
            cand --number-of-lines 'Number of lines of output to show. (Default is terminal_height - 10)'
            cand -X 'Exclude any file or directory with this path'
            cand --ignore-directory 'Exclude any file or directory with this path'
            cand -I 'Exclude any file or directory with a regex matching that listed in this file, the file entries will be added to the ignore regexs provided by --invert_filter'
            cand --ignore-all-in-file 'Exclude any file or directory with a regex matching that listed in this file, the file entries will be added to the ignore regexs provided by --invert_filter'
            cand -z 'Minimum size file to include in output'
            cand --min-size 'Minimum size file to include in output'
            cand -v 'Exclude filepaths matching this regex. To ignore png files type: -v "\.png$" '
            cand --invert-filter 'Exclude filepaths matching this regex. To ignore png files type: -v "\.png$" '
            cand -e 'Only include filepaths matching this regex. For png files type: -e "\.png$" '
            cand --filter 'Only include filepaths matching this regex. For png files type: -e "\.png$" '
            cand -w 'Specify width of output overriding the auto detection of terminal width'
            cand --terminal_width 'Specify width of output overriding the auto detection of terminal width'
            cand -o 'Changes output display size. si will print sizes in powers of 1000. b k m g t kb mb gb tb will print the whole tree in that size.'
            cand --output-format 'Changes output display size. si will print sizes in powers of 1000. b k m g t kb mb gb tb will print the whole tree in that size.'
            cand -S 'Specify memory to use as stack size - use if you see: ''fatal runtime error: stack overflow'' (default low memory=1048576, high memory=1073741824)'
            cand --stack-size 'Specify memory to use as stack size - use if you see: ''fatal runtime error: stack overflow'' (default low memory=1048576, high memory=1073741824)'
            cand -M '+/-n matches files modified more/less than n days ago , and n matches files modified exactly n days ago, days are rounded down.That is +n => (−∞, curr−(n+1)), n => [curr−(n+1), curr−n), and -n => (𝑐𝑢𝑟𝑟−𝑛, +∞)'
            cand --mtime '+/-n matches files modified more/less than n days ago , and n matches files modified exactly n days ago, days are rounded down.That is +n => (−∞, curr−(n+1)), n => [curr−(n+1), curr−n), and -n => (𝑐𝑢𝑟𝑟−𝑛, +∞)'
            cand -A 'just like -mtime, but based on file access time'
            cand --atime 'just like -mtime, but based on file access time'
            cand -y 'just like -mtime, but based on file change time'
            cand --ctime 'just like -mtime, but based on file change time'
            cand --files0-from 'run dust on NUL-terminated file names specified in file; if argument is -, then read names from standard input'
            cand --collapse 'Keep these directories collapsed'
            cand -m 'Directory ''size'' is max filetime of child files instead of disk size. while a/c/m for last accessed/changed/modified time'
            cand --filetime 'Directory ''size'' is max filetime of child files instead of disk size. while a/c/m for last accessed/changed/modified time'
            cand -p 'Subdirectories will not have their path shortened'
            cand --full-paths 'Subdirectories will not have their path shortened'
            cand -L 'dereference sym links - Treat sym links as directories and go into them'
            cand --dereference-links 'dereference sym links - Treat sym links as directories and go into them'
            cand -x 'Only count the files and directories on the same filesystem as the supplied directory'
            cand --limit-filesystem 'Only count the files and directories on the same filesystem as the supplied directory'
            cand -s 'Use file length instead of blocks'
            cand --apparent-size 'Use file length instead of blocks'
            cand -r 'Print tree upside down (biggest highest)'
            cand --reverse 'Print tree upside down (biggest highest)'
            cand -c 'No colors will be printed (Useful for commands like: watch)'
            cand --no-colors 'No colors will be printed (Useful for commands like: watch)'
            cand -C 'Force colors print'
            cand --force-colors 'Force colors print'
            cand -b 'No percent bars or percentages will be displayed'
            cand --no-percent-bars 'No percent bars or percentages will be displayed'
            cand -B 'percent bars moved to right side of screen'
            cand --bars-on-right 'percent bars moved to right side of screen'
            cand -R 'For screen readers. Removes bars. Adds new column: depth level (May want to use -p too for full path)'
            cand --screen-reader 'For screen readers. Removes bars. Adds new column: depth level (May want to use -p too for full path)'
            cand --skip-total 'No total row will be displayed'
            cand -f 'Directory ''size'' is number of child files instead of disk size'
            cand --filecount 'Directory ''size'' is number of child files instead of disk size'
            cand -i 'Do not display hidden files'
            cand --ignore_hidden 'Do not display hidden files'
            cand -t 'show only these file types'
            cand --file_types 'show only these file types'
            cand -P 'Disable the progress indication.'
            cand --no-progress 'Disable the progress indication.'
            cand --print-errors 'Print path with errors.'
            cand -D 'Only directories will be displayed.'
            cand --only-dir 'Only directories will be displayed.'
            cand -F 'Only files will be displayed. (Finds your largest files)'
            cand --only-file 'Only files will be displayed. (Finds your largest files)'
            cand -j 'Output the directory tree as json to the current directory'
            cand --output-json 'Output the directory tree as json to the current directory'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
    ]
    $completions[$command]
}
