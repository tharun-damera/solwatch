import { accountData, accountIndexStatus } from "../api/api";

export async function searchAddress(
  address,
  setAddress,
  setIndexed,
  setAccount
) {
  setAddress(address);
  let result = await accountIndexStatus(address);
  window.history.replaceState({}, "", `?address=${address}`);
  if (result.indexed) {
    let acc = await accountData(address);
    setIndexed(true);
    setAccount(acc);
  } else {
    setIndexed(false);
    setAccount(null);
  }
}
