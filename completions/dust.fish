complete -c dust -s d -l depth -d 'Depth to show' -r
complete -c dust -s n -l number-of-lines -d 'Number of lines of output to show. (Default is terminal_height - 10)' -r
complete -c dust -s X -l ignore-directory -d 'Exclude any file or directory with this name' -r
complete -c dust -s z -l min-size -d 'Minimum size file to include in output' -r
complete -c dust -s v -l invert-filter -d 'Exclude filepaths matching this regex. To ignore png files type: -v "\\.png$" ' -r
complete -c dust -s e -l filter -d 'Only include filepaths matching this regex. For png files type: -e "\\.png$" ' -r
complete -c dust -s w -l terminal_width -d 'Specify width of output overriding the auto detection of terminal width' -r
complete -c dust -s h -l help -d 'Print help information'
complete -c dust -s V -l version -d 'Print version information'
complete -c dust -s p -l full-paths -d 'Subdirectories will not have their path shortened'
complete -c dust -s L -l dereference-links -d 'dereference sym links - Treat sym links as directories and go into them'
complete -c dust -s x -l limit-filesystem -d 'Only count the files and directories on the same filesystem as the supplied directory'
complete -c dust -s s -l apparent-size -d 'Use file length instead of blocks'
complete -c dust -s r -l reverse -d 'Print tree upside down (biggest highest)'
complete -c dust -s c -l no-colors -d 'No colors will be printed (Useful for commands like: watch)'
complete -c dust -s b -l no-percent-bars -d 'No percent bars or percentages will be displayed'
complete -c dust -s R -l screen-reader -d 'For screen readers. Removes bars. Adds new column: depth level (May want to use -p too for full path)'
complete -c dust -l skip-total -d 'No total row will be displayed'
complete -c dust -s f -l filecount -d 'Directory \'size\' is number of child files/dirs not disk size'
complete -c dust -s i -l ignore_hidden -d 'Do not display hidden files'
complete -c dust -s t -l file_types -d 'show only these file types'
complete -c dust -s H -l si -d 'print sizes in powers of 1000 (e.g., 1.1G)'
complete -c dust -s P -l no-progress -d 'Disable the progress indication.'
complete -c dust -s D -l only-dir -d 'Only directories will be displayed.'
complete -c dust -s F -l only-file -d 'Only files will be displayed. (Finds your largest files)'
