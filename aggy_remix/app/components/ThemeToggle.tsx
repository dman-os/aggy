import { useAtom } from "jotai";
import React from "react";

import { Tooltip, Toggle } from ".";
import { themeAtom, Theme } from "~/utils/theme";

export const ThemeToggle = React.forwardRef<HTMLButtonElement, React.ComponentPropsWithRef<"button">>(
  function ThemeToggle(props, ref) {
    const [theme, setTheme] = useAtom(themeAtom);

    return (
      <Tooltip.Provider>
        <Tooltip.Root>
          <Tooltip.Trigger asChild>
            <Toggle.Root
              {...props}
              ref={ref}
              aria-label="Theme toggle"
              onPressedChange={() => setTheme(theme === Theme.LIGHT ? Theme.DARK : Theme.LIGHT)}
            >
              {theme === Theme.LIGHT ?
                <div className="i-solar-sun-2-outline" /> :
                <div className="i-solar-moon-stars-bold-duotone" />}
            </Toggle.Root>
          </Tooltip.Trigger>
          <Tooltip.Portal>
            <Tooltip.Content className="p-1 mat-secondary rounded-1" sideOffset={5}>
              Toggle Theme
              <Tooltip.Arrow className="TooltipArrow" />
            </Tooltip.Content>
          </Tooltip.Portal>
        </Tooltip.Root>
      </Tooltip.Provider>
    );
  }
);

