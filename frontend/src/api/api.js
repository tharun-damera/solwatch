const BASE_URL = import.meta.env.VITE_API_URL;

export async function accountIndexStatus(address) {
  let res = await fetch(`${BASE_URL}/api/accounts/${address}/status`, {
    method: "GET",
  });
  let resp = await res.json();
  return resp;
}

export async function transactionHistory(address, skip, limit) {
  let res = await fetch(
    `${BASE_URL}/api/accounts/${address}/signatures?skip=${skip}&limit=${limit}`,
    {
      method: "GET",
    }
  );
  let resp = await res.json();
  return resp;
}
