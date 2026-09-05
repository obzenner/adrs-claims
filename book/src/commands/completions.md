# completions

Generate shell completions for tab-completion support.

## Usage

```
cladrs completions <SHELL>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `<SHELL>` | Shell to generate completions for |

## Supported Shells

| Shell | Description |
|-------|-------------|
| `bash` | Bourne Again Shell |
| `zsh` | Z Shell |
| `fish` | Fish Shell |
| `powershell` | PowerShell |
| `elvish` | Elvish Shell |

## Installation

### Bash

```sh
# Generate completions
cladrs completions bash > ~/.bash_completion.d/adrs

# Add to ~/.bashrc
source ~/.bash_completion.d/adrs
```

### Zsh

```sh
# Generate completions
cladrs completions zsh > ~/.zfunc/_adrs

# Add to ~/.zshrc (before compinit)
fpath=(~/.zfunc $fpath)
autoload -Uz compinit && compinit
```

### Fish

```sh
# Generate completions (loaded automatically)
cladrs completions fish > ~/.config/fish/completions/adrs.fish
```

### PowerShell

```powershell
# Generate completions
cladrs completions powershell > _adrs.ps1

# Add to your PowerShell profile
. _adrs.ps1
```

### Elvish

```sh
# Generate completions
cladrs completions elvish > ~/.elvish/lib/adrs.elv

# Add to ~/.elvish/rc.elv
use adrs
```

## Examples

After installation, you can use tab completion:

```sh
# Complete commands
adrs <TAB>
init  new  edit  list  search  link  status  ...

# Complete options
cladrs new --<TAB>
--format  --variant  --tags  --supersedes  --no-edit  ...

# Complete formats
cladrs new --format <TAB>
nygard  madr
```

## Homebrew Users

If you installed via Homebrew, completions may already be configured. Check with:

```sh
# Bash (Homebrew)
$(brew --prefix)/etc/bash_completion.d/adrs

# Zsh (Homebrew)
$(brew --prefix)/share/zsh/site-functions/_adrs
```
