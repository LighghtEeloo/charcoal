
use builtin;
use str;

set edit:completion:arg-completer[charcoal] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'charcoal'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'charcoal'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand query 'Query words from online or offline'
            cand edit 'Edit the configuration file'
            cand cache 'Cache commands'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'charcoal;query'= {
            cand --speak-as 'Whether to speak aloud'
            cand --concise-as 'Whether to be concise'
            cand -s 'Speak aloud'
            cand --speak 'Speak aloud'
            cand -q 'Mute (overloads speak)'
            cand --mute 'Mute (overloads speak)'
            cand --refresh 'Whether to refresh cache'
            cand -c 'Be concise'
            cand --concise 'Be concise'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'charcoal;edit'= {
            cand --reset 'A fresh start'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'charcoal;cache'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand show 'Show cache location'
            cand clean 'Clean cache'
            cand import 'Import'
            cand export 'Export'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'charcoal;cache;show'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'charcoal;cache;clean'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'charcoal;cache;import'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'charcoal;cache;export'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'charcoal;cache;help'= {
            cand show 'Show cache location'
            cand clean 'Clean cache'
            cand import 'Import'
            cand export 'Export'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'charcoal;cache;help;show'= {
        }
        &'charcoal;cache;help;clean'= {
        }
        &'charcoal;cache;help;import'= {
        }
        &'charcoal;cache;help;export'= {
        }
        &'charcoal;cache;help;help'= {
        }
        &'charcoal;help'= {
            cand query 'Query words from online or offline'
            cand edit 'Edit the configuration file'
            cand cache 'Cache commands'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'charcoal;help;query'= {
        }
        &'charcoal;help;edit'= {
        }
        &'charcoal;help;cache'= {
            cand show 'Show cache location'
            cand clean 'Clean cache'
            cand import 'Import'
            cand export 'Export'
        }
        &'charcoal;help;cache;show'= {
        }
        &'charcoal;help;cache;clean'= {
        }
        &'charcoal;help;cache;import'= {
        }
        &'charcoal;help;cache;export'= {
        }
        &'charcoal;help;help'= {
        }
    ]
    $completions[$command]
}
