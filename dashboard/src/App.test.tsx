import { render, screen } from "@testing-library/react";
import App from "./App";

describe("App", () => {
  it("renders the Phase 0 dashboard scaffold", () => {
    render(<App />);

    expect(screen.getByRole("heading", { name: "EdgeFleet" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Fleet Inventory" })).toBeInTheDocument();
    expect(screen.getAllByText("edge-042")).toHaveLength(2);
    expect(screen.getByText("Live Telemetry")).toBeInTheDocument();
  });
});
