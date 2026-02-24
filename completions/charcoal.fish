# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_charcoal_global_optspecs
	string join \n h/help V/version
end

function __fish_charcoal_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_charcoal_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_charcoal_using_subcommand
	set -l cmd (__fish_charcoal_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c charcoal -n "__fish_charcoal_needs_command" -s h -l help -d 'Print help'
complete -c charcoal -n "__fish_charcoal_needs_command" -s V -l version -d 'Print version'
complete -c charcoal -n "__fish_charcoal_needs_command" -f -a "query" -d 'Query words from online or offline'
complete -c charcoal -n "__fish_charcoal_needs_command" -f -a "edit" -d 'Edit the configuration file'
complete -c charcoal -n "__fish_charcoal_needs_command" -f -a "cache" -d 'Cache commands'
complete -c charcoal -n "__fish_charcoal_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c charcoal -n "__fish_charcoal_using_subcommand query" -l speak-as -d 'Whether to speak aloud' -r -f -a "true\t'True'
false\t'False'
flip\t'Flip'"
complete -c charcoal -n "__fish_charcoal_using_subcommand query" -l concise-as -d 'Whether to be concise' -r -f -a "true\t'True'
false\t'False'
flip\t'Flip'"
complete -c charcoal -n "__fish_charcoal_using_subcommand query" -s s -l speak -d 'Speak aloud'
complete -c charcoal -n "__fish_charcoal_using_subcommand query" -s q -l mute -d 'Mute (overloads speak)'
complete -c charcoal -n "__fish_charcoal_using_subcommand query" -l refresh -d 'Whether to refresh cache'
complete -c charcoal -n "__fish_charcoal_using_subcommand query" -s c -l concise -d 'Be concise'
complete -c charcoal -n "__fish_charcoal_using_subcommand query" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c charcoal -n "__fish_charcoal_using_subcommand edit" -l reset -d 'A fresh start'
complete -c charcoal -n "__fish_charcoal_using_subcommand edit" -s h -l help -d 'Print help'
complete -c charcoal -n "__fish_charcoal_using_subcommand cache; and not __fish_seen_subcommand_from show clean import export help" -s h -l help -d 'Print help'
complete -c charcoal -n "__fish_charcoal_using_subcommand cache; and not __fish_seen_subcommand_from show clean import export help" -f -a "show" -d 'Show cache location'
complete -c charcoal -n "__fish_charcoal_using_subcommand cache; and not __fish_seen_subcommand_from show clean import export help" -f -a "clean" -d 'Clean cache'
complete -c charcoal -n "__fish_charcoal_using_subcommand cache; and not __fish_seen_subcommand_from show clean import export help" -f -a "import" -d 'Import'
complete -c charcoal -n "__fish_charcoal_using_subcommand cache; and not __fish_seen_subcommand_from show clean import export help" -f -a "export" -d 'Export'
complete -c charcoal -n "__fish_charcoal_using_subcommand cache; and not __fish_seen_subcommand_from show clean import export help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c charcoal -n "__fish_charcoal_using_subcommand cache; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c charcoal -n "__fish_charcoal_using_subcommand cache; and __fish_seen_subcommand_from clean" -s h -l help -d 'Print help'
complete -c charcoal -n "__fish_charcoal_using_subcommand cache; and __fish_seen_subcommand_from import" -s h -l help -d 'Print help'
complete -c charcoal -n "__fish_charcoal_using_subcommand cache; and __fish_seen_subcommand_from export" -s h -l help -d 'Print help'
complete -c charcoal -n "__fish_charcoal_using_subcommand cache; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show cache location'
complete -c charcoal -n "__fish_charcoal_using_subcommand cache; and __fish_seen_subcommand_from help" -f -a "clean" -d 'Clean cache'
complete -c charcoal -n "__fish_charcoal_using_subcommand cache; and __fish_seen_subcommand_from help" -f -a "import" -d 'Import'
complete -c charcoal -n "__fish_charcoal_using_subcommand cache; and __fish_seen_subcommand_from help" -f -a "export" -d 'Export'
complete -c charcoal -n "__fish_charcoal_using_subcommand cache; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c charcoal -n "__fish_charcoal_using_subcommand help; and not __fish_seen_subcommand_from query edit cache help" -f -a "query" -d 'Query words from online or offline'
complete -c charcoal -n "__fish_charcoal_using_subcommand help; and not __fish_seen_subcommand_from query edit cache help" -f -a "edit" -d 'Edit the configuration file'
complete -c charcoal -n "__fish_charcoal_using_subcommand help; and not __fish_seen_subcommand_from query edit cache help" -f -a "cache" -d 'Cache commands'
complete -c charcoal -n "__fish_charcoal_using_subcommand help; and not __fish_seen_subcommand_from query edit cache help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c charcoal -n "__fish_charcoal_using_subcommand help; and __fish_seen_subcommand_from cache" -f -a "show" -d 'Show cache location'
complete -c charcoal -n "__fish_charcoal_using_subcommand help; and __fish_seen_subcommand_from cache" -f -a "clean" -d 'Clean cache'
complete -c charcoal -n "__fish_charcoal_using_subcommand help; and __fish_seen_subcommand_from cache" -f -a "import" -d 'Import'
complete -c charcoal -n "__fish_charcoal_using_subcommand help; and __fish_seen_subcommand_from cache" -f -a "export" -d 'Export'
