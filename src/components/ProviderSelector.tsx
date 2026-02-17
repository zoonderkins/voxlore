import { Select } from "./ui";

interface ProviderOption {
  value: string;
  label: string;
}

interface ProviderSelectorProps {
  options: readonly ProviderOption[];
  value: string;
  onChange: (value: string) => void;
  label?: string;
}

export function ProviderSelector({ options, value, onChange, label }: ProviderSelectorProps) {
  return (
    <Select
      label={label}
      options={[...options]}
      value={value}
      onChange={onChange}
    />
  );
}
