complete -c dust -s d -l depth -d 'Depth to show' -r
complete -c dust -s T -l threads -d 'Number of threads to use' -r
complete -c dust -s n -l number-of-lines -d 'Number of lines of output to show. (Default is terminal_height - 10)' -r
complete -c dust -s X -l ignore-directory -d 'Exclude any file or directory with this name' -r -F
complete -c dust -s I -l ignore-all-in-file -d 'Exclude any file or directory with a regex matching that listed in this file, the file entries will be added to the ignore regexs provided by --invert_filter' -r -F
complete -c dust -s z -l min-size -d 'Minimum size file to include in output' -r
complete -c dust -s v -l invert-filter -d 'Exclude filepaths matching this regex. To ignore png files type: -v "\\.png$" ' -r
complete -c dust -s e -l filter -d 'Only include filepaths matching this regex. For png files type: -e "\\.png$" ' -r
complete -c dust -s w -l terminal_width -d 'Specify width of output overriding the auto detection of terminal width' -r
complete -c dust -s o -l output-format -d 'Changes output display size. si will print sizes in powers of 1000. b k m g t kb mb gb tb will print the whole tree in that size.' -r -f -a "{si	'',b	'',k	'',m	'',g	'',t	'',kb	'',mb	'',gb	'',tb	''}"
complete -c dust -s S -l stack-size -d 'Specify memory to use as stack size - use if you see: \'fatal runtime error: stack overflow\' (default low memory=1048576, high memory=1073741824)' -r
complete -c dust -s M -l mtime -d '+/-n matches files modified more/less than n days ago , and n matches files modified exactly n days ago, days are rounded down.That is +n => (−∞, curr−(n+1)), n => [curr−(n+1), curr−n), and -n => (𝑐𝑢𝑟𝑟−𝑛, +∞)' -r
complete -c dust -s A -l atime -d 'just like -mtime, but based on file access time' -r
complete -c dust -s y -l ctime -d 'just like -mtime, but based on file change time' -r
complete -c dust -l files0-from -d 'run dust on NUL-terminated file names specified in file; if argument is -, then read names from standard input' -r -F
complete -c dust -s m -l filetime -d 'Directory \'size\' is max filetime of child files instead of disk size. while a/c/m for last accessed/changed/modified time' -r -f -a "{a	'',c	'',m	''}"
complete -c dust -s p -l full-paths -d 'Subdirectories will not have their path shortened'
complete -c dust -s L -l dereference-links -d 'dereference sym links - Treat sym links as directories and go into them'
complete -c dust -s x -l limit-filesystem -d 'Only count the files and directories on the same filesystem as the supplied directory'
complete -c dust -s s -l apparent-size -d 'Use file length instead of blocks'
complete -c dust -s r -l reverse -d 'Print tree upside down (biggest highest)'
complete -c dust -s c -l no-colors -d 'No colors will be printed (Useful for commands like: watch)'
complete -c dust -s C -l force-colors -d 'Force colors print'
complete -c dust -s b -l no-percent-bars -d 'No percent bars or percentages will be displayed'
complete -c dust -s B -l bars-on-right -d 'percent bars moved to right side of screen'
complete -c dust -s R -l screen-reader -d 'For screen readers. Removes bars. Adds new column: depth level (May want to use -p too for full path)'
complete -c dust -l skip-total -d 'No total row will be displayed'
complete -c dust -s f -l filecount -d 'Directory \'size\' is number of child files instead of disk size'
complete -c dust -s i -l ignore_hidden -d 'Do not display hidden files'
complete -c dust -s t -l file_types -d 'show only these file types'
complete -c dust -s P -l no-progress -d 'Disable the progress indication.'
complete -c dust -l print-errors -d 'Print path with errors.'
complete -c dust -s D -l only-dir -d 'Only directories will be displayed.'
complete -c dust -s F -l only-file -d 'Only files will be displayed. (Finds your largest files)'
complete -c dust -s j -l output-json -d 'Output the directory tree as json to the current directory'
complete -c dust -s h -l help -d 'Print help'
complete -c dust -s V -l version -d 'Print version'
