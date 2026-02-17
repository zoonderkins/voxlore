import { useState } from "react";
import { StepIndicator, Button } from "../../components/ui";
import { useSettingsStore } from "../../stores/useSettingsStore";
import { VoiceProviderStep } from "./steps/VoiceProviderStep";
import { TextEnhancementStep } from "./steps/TextEnhancementStep";
import { PermissionsStep } from "./steps/PermissionsStep";
import { HowToUseStep } from "./steps/HowToUseStep";
import { CompletionStep } from "./steps/CompletionStep";

const TOTAL_STEPS = 5;

const steps = [
  VoiceProviderStep,
  TextEnhancementStep,
  PermissionsStep,
  HowToUseStep,
  CompletionStep,
];

export function SetupWizard() {
  const [currentStep, setCurrentStep] = useState(0);
  const { updateSettings } = useSettingsStore();

  const isFirst = currentStep === 0;
  const isLast = currentStep === TOTAL_STEPS - 1;

  const handleNext = () => {
    if (isLast) {
      updateSettings({ setupCompleted: true });
    } else {
      setCurrentStep((s) => Math.min(s + 1, TOTAL_STEPS - 1));
    }
  };

  const handleBack = () => {
    setCurrentStep((s) => Math.max(s - 1, 0));
  };

  const StepComponent = steps[currentStep]!;

  return (
    <div className="flex h-full flex-col bg-bg-primary">
      {/* Step dots */}
      <div className="py-4">
        <StepIndicator totalSteps={TOTAL_STEPS} currentStep={currentStep} />
      </div>

      {/* Step content */}
      <div className="flex-1 overflow-y-auto px-6">
        <StepComponent />
      </div>

      {/* Navigation */}
      <div className="flex items-center justify-between border-t border-border px-6 py-4">
        {isFirst ? (
          <div />
        ) : (
          <Button variant="ghost" onClick={handleBack}>
            Back
          </Button>
        )}
        <Button variant="primary" onClick={handleNext}>
          {isLast ? "Done" : "Continue"}
        </Button>
      </div>
    </div>
  );
}
