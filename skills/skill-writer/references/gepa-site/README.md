# GEPA Site Mirror

Local mirror path:

```text
skill-writer/references/gepa-site/gepa-ai.github.io/gepa/
```

Source site:

```text
https://gepa-ai.github.io/gepa/
```

Scrape command used on 2026-06-03:

```bash
mkdir -p "/home/lewis/.agents/skills/skill-writer/references/gepa-site" && rtk wget --mirror --page-requisites --adjust-extension --convert-links --no-parent --domains=gepa-ai.github.io --wait=0.2 --directory-prefix="/home/lewis/.agents/skills/skill-writer/references/gepa-site" "https://gepa-ai.github.io/gepa/"
```

Scope:

- Included pages and assets hosted under `gepa-ai.github.io/gepa/`.
- Excluded external links such as GitHub repositories, Discord, Slack, arXiv, notebooks, and vendor documentation.
- Wget reported `Downloaded: 204 files, 39M` and converted links in 120 files.
- Some internal source-code links returned 404 during scraping; these were not external misses.

Key local entry points:

```text
gepa-ai.github.io/gepa/index.html
gepa-ai.github.io/gepa/guides/index.html
gepa-ai.github.io/gepa/guides/adapters/index.html
gepa-ai.github.io/gepa/guides/gskill/index.html
gepa-ai.github.io/gepa/guides/candidate-selection/index.html
gepa-ai.github.io/gepa/guides/acceptance-criterion/index.html
gepa-ai.github.io/gepa/guides/batch-sampling/index.html
gepa-ai.github.io/gepa/guides/callbacks/index.html
gepa-ai.github.io/gepa/guides/cost-tracking/index.html
gepa-ai.github.io/gepa/guides/experiment-tracking/index.html
gepa-ai.github.io/gepa/api/core/GEPAAdapter/index.html
gepa-ai.github.io/gepa/api/core/EvaluationBatch/index.html
gepa-ai.github.io/gepa/api/optimize_anything/optimize_anything/index.html
gepa-ai.github.io/gepa/api/proposers/ReflectiveMutationProposer/index.html
gepa-ai.github.io/gepa/blog/2026/02/18/automatically-learning-skills-for-coding-agents/index.html
gepa-ai.github.io/gepa/blog/2026/02/18/introducing-optimize-anything/index.html
```

Operational guidance:

- Use this mirror as a reference library, not as prompt bulk.
- Prefer `references/gepa-adapter-guide.md` for adapter mechanics.
- Prefer `references/gepa-skill-optimization.md` for applying GEPA to local skills.
- Re-scrape only when the user asks for a refresh or when the local mirror is stale for the task.
