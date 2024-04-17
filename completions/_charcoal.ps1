
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'charcoal' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'charcoal'
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
        'charcoal' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', 'V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('query', 'query', [CompletionResultType]::ParameterValue, 'Query words from online or offline')
            [CompletionResult]::new('edit', 'edit', [CompletionResultType]::ParameterValue, 'Edit the configuration file')
            [CompletionResult]::new('cache', 'cache', [CompletionResultType]::ParameterValue, 'Cache commands')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'charcoal;query' {
            [CompletionResult]::new('--speak-as', 'speak-as', [CompletionResultType]::ParameterName, 'Whether to speak aloud')
            [CompletionResult]::new('--concise-as', 'concise-as', [CompletionResultType]::ParameterName, 'Whether to be concise')
            [CompletionResult]::new('-s', 's', [CompletionResultType]::ParameterName, 'Speak aloud')
            [CompletionResult]::new('--speak', 'speak', [CompletionResultType]::ParameterName, 'Speak aloud')
            [CompletionResult]::new('-q', 'q', [CompletionResultType]::ParameterName, 'Mute (overloads speak)')
            [CompletionResult]::new('--mute', 'mute', [CompletionResultType]::ParameterName, 'Mute (overloads speak)')
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'Be concise')
            [CompletionResult]::new('--concise', 'concise', [CompletionResultType]::ParameterName, 'Be concise')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'charcoal;edit' {
            [CompletionResult]::new('--reset', 'reset', [CompletionResultType]::ParameterName, 'A fresh start')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'charcoal;cache' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Show cache location')
            [CompletionResult]::new('clean', 'clean', [CompletionResultType]::ParameterValue, 'Clean cache')
            [CompletionResult]::new('import', 'import', [CompletionResultType]::ParameterValue, 'Import')
            [CompletionResult]::new('export', 'export', [CompletionResultType]::ParameterValue, 'Export')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'charcoal;cache;show' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'charcoal;cache;clean' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'charcoal;cache;import' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'charcoal;cache;export' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'charcoal;cache;help' {
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Show cache location')
            [CompletionResult]::new('clean', 'clean', [CompletionResultType]::ParameterValue, 'Clean cache')
            [CompletionResult]::new('import', 'import', [CompletionResultType]::ParameterValue, 'Import')
            [CompletionResult]::new('export', 'export', [CompletionResultType]::ParameterValue, 'Export')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'charcoal;cache;help;show' {
            break
        }
        'charcoal;cache;help;clean' {
            break
        }
        'charcoal;cache;help;import' {
            break
        }
        'charcoal;cache;help;export' {
            break
        }
        'charcoal;cache;help;help' {
            break
        }
        'charcoal;help' {
            [CompletionResult]::new('query', 'query', [CompletionResultType]::ParameterValue, 'Query words from online or offline')
            [CompletionResult]::new('edit', 'edit', [CompletionResultType]::ParameterValue, 'Edit the configuration file')
            [CompletionResult]::new('cache', 'cache', [CompletionResultType]::ParameterValue, 'Cache commands')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'charcoal;help;query' {
            break
        }
        'charcoal;help;edit' {
            break
        }
        'charcoal;help;cache' {
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Show cache location')
            [CompletionResult]::new('clean', 'clean', [CompletionResultType]::ParameterValue, 'Clean cache')
            [CompletionResult]::new('import', 'import', [CompletionResultType]::ParameterValue, 'Import')
            [CompletionResult]::new('export', 'export', [CompletionResultType]::ParameterValue, 'Export')
            break
        }
        'charcoal;help;cache;show' {
            break
        }
        'charcoal;help;cache;clean' {
            break
        }
        'charcoal;help;cache;import' {
            break
        }
        'charcoal;help;cache;export' {
            break
        }
        'charcoal;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
