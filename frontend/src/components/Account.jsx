import { useEffect } from "react";
import { Card, CardHeader, CardBody } from "./Card";

const BASE_URL = import.meta.env.VITE_API_URL;

export default function Account({ address, account, setAccount, setError }) {
  useEffect(() => {
    async function accountData(address) {
      try {
        const res = await fetch(`${BASE_URL}/api/accounts/${address}`, {
          method: "GET",
        });
        if (!res.ok) throw new Error("Something went wrong");
        const data = await res.json();
        setAccount(data);
      } catch (err) {
        setError(err.message);
      }
    }
    accountData(address);
  }, [address, setError, setAccount]);

  if (!account) return <></>;

  return (
    <>
      <Card>
        <CardHeader>Account Overview</CardHeader>
        <CardBody>
          <table>
            <tbody>
              <tr>
                <td>Address</td>
                <td className="mono responsive-td">{account._id}</td>
              </tr>
              <tr>
                <td>Balance (Lamports)</td>
                <td className="responsive-td">
                  <strong>◎{account.lamports ?? 0}</strong>
                </td>
              </tr>
              <tr>
                <td>Owner</td>
                <td className="responsive-td">{account.owner}</td>
              </tr>
              <tr>
                <td>Executable</td>
                <td className="responsive-td">
                  {account.executable ? "Yes" : "No"}
                </td>
              </tr>
              <tr>
                <td>Allocated Data Size</td>
                <td className="responsive-td">{account.data_length} byte(s)</td>
              </tr>
            </tbody>
          </table>
        </CardBody>
      </Card>
      <div className="horizontal-line"></div>
    </>
  );
}
