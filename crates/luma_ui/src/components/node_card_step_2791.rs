python3 -c '
import json
with open("/home/abdellah/.gemini/antigravity-cli/brain/6df659ea-a64c-4d8b-a7cc-ea6aed82f9ea/.system_generated/logs/transcript_full.jsonl") as f:
    for i, line in enumerate(f):
        if "fn node_card_scene(" in line:
            print(f"Line {i}")
            data = json.loads(line)
            for tc in data.get("tool_calls", []):
                args = tc.get("args", {})
                for k, v in args.items():
                    if isinstance(v, str) and "fn node_card_scene(" in v:
                        print(f"  key {k}: len={len(v)}")
                        with open(f"/home/abdellah/Univis-project/luma_editor/crates/luma_ui/src/components/node_card_step_{i}.rs", "w") as out:
                            out.write(v)
            c = data.get("content", "")
            if "fn node_card_scene(" in c:
                print(f"  content: len={len(c)}")
                with open(f"/home/abdellah/Univis-project/luma_editor/crates/luma_ui/src/components/node_card_content_{i}.rs", "w") as out:
                    out.write(c)
'