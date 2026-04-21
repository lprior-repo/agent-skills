# Skill Templates

## 1) Advisory Skill (Auto + Manual)

```yaml
---
name: <skill-name>
description: <what it does and when to use it>
argument-hint: [optional args]
allowed-tools:
  - Read
  - Grep
  - Glob
---
```

## 2) Manual-Only Workflow Skill

```yaml
---
name: <skill-name>
description: <workflow with side effects>
disable-model-invocation: true
argument-hint: [target]
allowed-tools:
  - Read
  - Bash
---
```

## 3) Background Context Skill (Model-Only)

```yaml
---
name: <skill-name>
description: <domain conventions and constraints>
user-invocable: false
allowed-tools:
  - Read
  - Grep
---
```

## 4) Forked Execution Skill

```yaml
---
name: <skill-name>
description: <task requiring isolated execution>
context: fork
agent: Explore
argument-hint: [topic]
allowed-tools:
  - Read
  - Glob
  - Grep
---
```
