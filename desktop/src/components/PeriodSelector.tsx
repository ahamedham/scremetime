import { Period } from "../lib/api";

const OPTIONS: { value: Period; label: string }[] = [
  { value: "today", label: "Today" },
  { value: "week", label: "Week" },
  { value: "month", label: "Month" },
  { value: "all", label: "All Time" },
];

interface Props {
  value: Period;
  onChange: (period: Period) => void;
}

export default function PeriodSelector({ value, onChange }: Props) {
  return (
    <div className="segmented-control">
      {OPTIONS.map((option) => (
        <button
          key={option.value}
          className={`segmented-control__option ${
            value === option.value ? "segmented-control__option--active" : ""
          }`}
          onClick={() => onChange(option.value)}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}
