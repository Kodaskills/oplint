/* OPLint landing page tweaks panel */
const { useEffect } = React;

const LANDING_DEFAULTS = /*EDITMODE-BEGIN*/{
  "theme": "dark",
  "accent": "purple",
  "anim": "medium",
  "density": "default"
}/*EDITMODE-END*/;

const ACCENTS = {
  purple: { a: "oklch(0.72 0.16 295)", a2: "oklch(0.78 0.13 320)" },
  indigo: { a: "oklch(0.66 0.18 265)", a2: "oklch(0.72 0.14 285)" },
  teal:   { a: "oklch(0.72 0.13 195)", a2: "oklch(0.78 0.10 215)" },
  amber:  { a: "oklch(0.78 0.14 60)",  a2: "oklch(0.82 0.12 80)" },
  rose:   { a: "oklch(0.68 0.18 15)",  a2: "oklch(0.72 0.16 350)" },
};
const ACCENTS_LIGHT = {
  purple: { a: "oklch(0.50 0.16 295)", a2: "oklch(0.55 0.15 320)" },
  indigo: { a: "oklch(0.46 0.18 265)", a2: "oklch(0.52 0.16 285)" },
  teal:   { a: "oklch(0.48 0.12 195)", a2: "oklch(0.55 0.10 215)" },
  amber:  { a: "oklch(0.55 0.16 60)",  a2: "oklch(0.62 0.14 80)" },
  rose:   { a: "oklch(0.52 0.18 15)",  a2: "oklch(0.55 0.16 350)" },
};

function LandingTweaks() {
  const [tweaks, setTweak] = useTweaks(LANDING_DEFAULTS);

  useEffect(() => {
    const root = document.documentElement;
    root.setAttribute('data-theme', tweaks.theme);
    root.setAttribute('data-anim', tweaks.anim === 'off' ? 'off' : 'on');
    root.setAttribute('data-density', tweaks.density);

    const isLight = tweaks.theme === 'light';
    const set = (isLight ? ACCENTS_LIGHT : ACCENTS)[tweaks.accent] || (isLight ? ACCENTS_LIGHT.purple : ACCENTS.purple);
    root.style.setProperty('--accent', set.a);
    root.style.setProperty('--accent-2', set.a2);
    root.style.setProperty('--accent-soft', set.a.replace(')', ' / 0.14)').replace('oklch(', 'oklch('));
    root.style.setProperty('--accent-line', set.a.replace(')', ' / 0.36)').replace('oklch(', 'oklch('));
  }, [tweaks]);

  return (
    <TweaksPanel title="Tweaks">
      <TweakSection label="Theme">
        <TweakRadio
          value={tweaks.theme}
          onChange={(v) => setTweak('theme', v)}
          options={[
            { value: 'dark', label: 'Dark' },
            { value: 'light', label: 'Light' },
          ]}
        />
      </TweakSection>

      <TweakSection label="Accent">
        <TweakSelect
          value={tweaks.accent}
          onChange={(v) => setTweak('accent', v)}
          options={[
            { value: 'purple', label: 'Purple (Obsidian)' },
            { value: 'indigo', label: 'Indigo' },
            { value: 'teal',   label: 'Teal' },
            { value: 'amber',  label: 'Amber' },
            { value: 'rose',   label: 'Rose' },
          ]}
        />
      </TweakSection>

      <TweakSection label="Animation">
        <TweakRadio
          value={tweaks.anim}
          onChange={(v) => setTweak('anim', v)}
          options={[
            { value: 'medium', label: 'On' },
            { value: 'off',    label: 'Off' },
          ]}
        />
      </TweakSection>

      <TweakSection label="Density">
        <TweakRadio
          value={tweaks.density}
          onChange={(v) => setTweak('density', v)}
          options={[
            { value: 'compact', label: 'Compact' },
            { value: 'default', label: 'Default' },
            { value: 'cozy',    label: 'Cozy' },
          ]}
        />
      </TweakSection>
    </TweaksPanel>
  );
}

const tweaksRoot = document.createElement('div');
document.body.appendChild(tweaksRoot);
ReactDOM.createRoot(tweaksRoot).render(<LandingTweaks />);
