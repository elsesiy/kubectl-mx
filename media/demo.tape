Output media/demo.gif
Set CursorBlink false
Set FontSize 16
Set Theme "Catppuccin Mocha"
Set Width 1400

Hide
Type "# Welcome to kubectl-mx - Run kubectl commands across multiple contexts with ease!"
Enter 2
Show
Sleep 2
Type "clear"
Enter

Hide
Type "# First, lets see the help"
Enter 2
Show
Sleep 2
Type "kubectl-mx --help"
Enter
Sleep 3
Type "clear"
Enter

Hide
Type "# Now, dry-run to see matching contexts for regex kind.* when retrieving all services in a cluster"
Enter 2
Show
Sleep 1
Type "kubectl-mx -d -r 'kind.*' -e get svc -A"
Enter
Sleep 3
Type "clear"
Enter

Hide
Type "# Executing the command"
Enter 2
Show
Sleep 1
Type "kubectl-mx -r 'kind.*' -e get svc -A"
Enter
Sleep 3
Type "clear"
Enter

Hide
Type "# JSON output processing with jq"
Enter 2
Show
Sleep 1
Type `kubectl-mx --json -r 'kind.*' -e get svc -A | jq -r '.results[] as $result | $result.output | fromjson | .items[] | "\($result.context) \(.metadata.namespace) \(.metadata.name) \(.spec.clusterIP)"'`
Enter
Sleep 3
Type "clear"
Enter

Hide
Type "# Demo complete! kubectl-mx allows for efficient parallel execution across contexts."
Enter
Show
Sleep 3
