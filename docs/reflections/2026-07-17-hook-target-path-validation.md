# Hook target-path validation

- Diagnosis: hook status previously treated a configuration as installed by searching for `CcTrafficLight` and the agent name in raw JSON.
- Consequence: a retained hook that still referenced a deleted prior installation was displayed as configured, even though its command could not run.
- Change: parse only CC Traffic Light-managed entries, extract each hook executable target, and report `needs_reinstall` when any target is unavailable.
- Safety: detection is read-only; users must explicitly select **Reinstall monitoring** to update hook configuration.
- Regression coverage: fixture tests cover both ChatGPT and Claude Code managed entries with missing targets.
