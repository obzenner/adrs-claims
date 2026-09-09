# cheatsheet

Show a quick reference for common ADR workflows.

## Usage

```
cladrs cheatsheet
```

Alias: `cladrs qr`

## Output

Displays a comprehensive quick reference covering:

- Getting started
- Creating ADRs
- Superseding and linking
- Managing status
- Viewing and searching
- Generating documentation
- Import/export
- Configuration

## Example Output

```
ADR Quick Reference
===================

GETTING STARTED
  cladrs init                    Initialize ADR repository
  cladrs --ng init               Initialize with NextGen mode (YAML frontmatter)

CREATING ADRS
  cladrs new "Title"             Create new ADR
  cladrs new --format madr       Use MADR 4.0.0 format
  cladrs new --variant minimal   Use minimal template
  cladrs --ng new -t tag1,tag2 "Title"  Add tags (NextGen mode)
  cladrs new --no-edit           Create without opening editor

SUPERSEDING AND LINKING
  cladrs new -s 2 "New title"    Supersede ADR #2
  cladrs link 3 Amends 1         Link ADR #3 amends #1

MANAGING STATUS
  cladrs status 1 accepted       Accept ADR #1
  cladrs status 2 deprecated     Deprecate ADR #2
  cladrs status 3 superseded --by 4  Mark #3 superseded by #4

VIEWING AND SEARCHING
  cladrs list                    List all ADRs
  cladrs list --status accepted  Filter by status
  cladrs search postgres         Search content
  cladrs search -t database      Search titles only

DOCUMENTATION
  cladrs generate toc            Generate table of contents
  cladrs generate graph          Generate Graphviz diagram
  cladrs generate book           Generate mdbook

IMPORT/EXPORT
  cladrs export json             Export to JSON-ADR
  cladrs import json file.json   Import from JSON-ADR

CONFIGURATION
  cladrs config                  Show current config
  cladrs doctor                  Check repository health

More: cladrs --help, adrs <command> --help
```

## Tips

- Run `cladrs cheatsheet` whenever you need a quick reminder
- Use `adrs <command> --help` for detailed command documentation
- Full documentation at [joshrotenberg.com/adrs](https://joshrotenberg.com/adrs/)
