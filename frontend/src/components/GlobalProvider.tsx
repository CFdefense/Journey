import { useState, type Dispatch, type SetStateAction } from "react";
import { GlobalContext } from "../helpers/global";

export type GlobalState = {
  authorized: boolean | null;
  setAuthorized: Dispatch<SetStateAction<boolean | null>>;
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function GlobalProvider({ children }: any) {
  const [authorized, setAuthorized] = useState<boolean | null>(null);
  return (
    <GlobalContext.Provider
      value={{
        authorized,
        setAuthorized
      }}
    >
      {children}
    </GlobalContext.Provider>
  );
}
