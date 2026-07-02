import { fireEvent, render, screen } from "@testing-library/react";
import { parse } from "yaml";
import { describe, expect, it, vi } from "vitest";
import { CreateRuleModal } from "./CreateRuleModal";

describe("CreateRuleModal", () => {
  it("serializes user input as data without changing the YAML structure", () => {
    const onSubmit = vi.fn();

    render(
      <CreateRuleModal
        isOpen
        onClose={vi.fn()}
        onSubmit={onSubmit}
        isSubmitting={false}
      />
    );

    fireEvent.change(screen.getByRole("textbox", { name: "Rule Name" }), {
      target: { value: 'Rule: "quoted"' },
    });
    fireEvent.change(screen.getByRole("textbox", { name: "Condition value" }), {
      target: { value: '*alias: "value"' },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save & Load Rule" }));

    expect(onSubmit).toHaveBeenCalledOnce();
    const payload = onSubmit.mock.calls[0][0];
    const rule = parse(payload.yaml_content);

    expect(rule).toMatchObject({
      id: "custom-rule-quoted",
      name: 'Rule: "quoted"',
      condition: {
        body_contains: {
          value: '*alias: "value"',
        },
      },
    });
  });
});
