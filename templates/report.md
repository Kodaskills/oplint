<img src="https://raw.githubusercontent.com/Kodaskills/oplint/refs/heads/main/docs/logo.svg" alt="oplint logo" width="128" height="128" />

# OPLint Compliance Report

**Generated:** {{ report_date }}  
**Files scanned:** {{ summary.total_files }}  
**Total violations:** {{ summary.total_violations }}

**Compliance:** {{summary.score}}/100{% if summary.partial_coverage %}*{% endif %} · [{{summary.grade}}] {{summary.grade_label}}

{% if summary.partial_coverage %}
> ⚠️ **Partial coverage** — score reflects only the active rules. Some guidelines are not checked because rules were explicitly disabled or restricted in your config.

{% endif %}
## Summary

| Severity | Count |
|----------|------:|
| 🔴 Errors | {{ summary.errors }} |
| ⚠️ Warnings | {{ summary.warnings }} |
| ℹ️ Info | {{ summary.infos }} |

**Performance:** {{ summary.duration_ms }} ms total · avg {{ summary.avg_file_ms }} ms/file · min {{ summary.min_file_ms }} ms · max {{ summary.max_file_ms }} ms

{% if errors.is_empty() && warnings.is_empty() && infos.is_empty() %}
---

✅ **No violations found.** Your plugin follows the Obsidian guidelines.
{% else %}
{% for group in errors %}
{% if loop.first %}
---

## {{ group.category }}

{% endif %}
### {{ group.rule_id }} — Line {{ group.line }} `{{ group.accuracy }}`

**File:** `{{ group.file }}`

{{ group.message }}

{% if group.source_code.is_some() %}
**Current:**
```
{{ group.source_code.as_deref().unwrap() }}
```
{% endif %}
{% if group.suggestion.is_some() %}
**Suggested:**
```
{{ group.suggestion.as_deref().unwrap() }}
```
{% endif %}
{% if group.accuracy_note.is_some() %}
> *Accuracy note: {{ group.accuracy_note.as_deref().unwrap() }}*
{% endif %}
{% if group.reference.is_some() %}
**Reference:** [Obsidian guidelines]({{ group.reference.as_deref().unwrap() }})
{% endif %}
{% endfor %}
{% for group in warnings %}
{% if loop.first %}
---

## {{ group.category }}

{% endif %}
### {{ group.rule_id }} — Line {{ group.line }} `{{ group.accuracy }}`

**File:** `{{ group.file }}`

{{ group.message }}

{% if group.source_code.is_some() %}
**Current:**
```
{{ group.source_code.as_deref().unwrap() }}
```
{% endif %}
{% if group.suggestion.is_some() %}
**Suggested:**
```
{{ group.suggestion.as_deref().unwrap() }}
```
{% endif %}
{% if group.accuracy_note.is_some() %}
> *Accuracy note: {{ group.accuracy_note.as_deref().unwrap() }}*
{% endif %}
{% if group.reference.is_some() %}
**Reference:** [Obsidian guidelines]({{ group.reference.as_deref().unwrap() }})
{% endif %}
{% endfor %}
{% for group in infos %}
{% if loop.first %}
---

## {{ group.category }}

{% endif %}
### {{ group.rule_id }} — Line {{ group.line }} `{{ group.accuracy }}`

**File:** `{{ group.file }}`

{{ group.message }}

{% if group.source_code.is_some() %}
**Current:**
```
{{ group.source_code.as_deref().unwrap() }}
```
{% endif %}
{% if group.suggestion.is_some() %}
**Suggested:**
```
{{ group.suggestion.as_deref().unwrap() }}
```
{% endif %}
{% if group.accuracy_note.is_some() %}
> *Accuracy note: {{ group.accuracy_note.as_deref().unwrap() }}*
{% endif %}
{% if group.reference.is_some() %}
**Reference:** [Obsidian guidelines]({{ group.reference.as_deref().unwrap() }})
{% endif %}
{% endfor %}
{% endif %}
