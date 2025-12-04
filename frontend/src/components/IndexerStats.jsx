import { useEffect, useState } from "react";
import { Card, CardHeader, CardBody } from "./Card";

const BASE_URL = import.meta.env.VITE_API_URL;

export default function IndexerStats({
  address,
  indexed,
  setAccount,
  setTxnsIndexed,
  setError,
}) {
  const [state, setState] = useState("⏸️ Idle");
  const [accountFetched, setAccountFetched] = useState(true);
  const [signatureStats, setSignatureStats] = useState({ total: 0 });
  const [txnStats, setTxnStats] = useState({ total: 0 });
  const [err, setErr] = useState(null);

  useEffect(() => {
    async function get_idle_stats() {
      try {
        const res = await fetch(
          `${BASE_URL}/api/accounts/${address}/indexer/stats`,
          {
            method: "GET",
          }
        );
        const data = await res.json();
        setSignatureStats({ total: data.signatures });
        setTxnStats({ total: data.transactions });
      } catch (err) {
        setError(err.message);
        setErr(err.message);
      }
    }
    const custom_state = indexed ? "🔁 Syncing" : "▶️ Running";

    if (indexed) {
      get_idle_stats();
    } else {
      const url = `${BASE_URL}/api/accounts/${address}/index/sse`;
      const sse = new EventSource(url);
      sse.addEventListener("started", () => {
        setState(custom_state);
        setAccountFetched(false);
      });
      sse.addEventListener("account-data", (e) => {
        const data = JSON.parse(e.data);
        console.log(data);
        setAccountFetched(true);
        setAccount(data);
      });
      sse.addEventListener("signatures-fetched", (e) => {
        const data = JSON.parse(e.data);
        setTxnStats(data);
      });
      sse.addEventListener("transactions-fetched", (e) => {
        const data = JSON.parse(e.data);
        setTxnsIndexed(data.total);
        setSignatureStats(data);
      });
      sse.addEventListener("error", (e) => {
        sse.close();
        setErr(e.data);
        setError(e.data);
      });
      sse.addEventListener("close", () => {
        sse.close();
        setState("✅ Completed");
      });
      return () => {
        sse.close();
      };
    }
  }, [address, indexed, setAccount, setTxnsIndexed, setError]);

  if (err) return <></>;

  return (
    <>
      <Card>
        <CardHeader>Indexer Stats</CardHeader>
        <CardBody>
          <table>
            <tbody>
              <tr>
                <td>State</td>
                <td className="responsive-td">{state}</td>
              </tr>
              <tr>
                <td>Account Fetched</td>
                <td className="responsive-td">
                  {accountFetched ? "✅" : "❌"}
                </td>
              </tr>
              <tr>
                <td>Signatures</td>
                <td className="responsive-td">
                  <span>
                    {signatureStats.total}{" "}
                    {signatureStats.fetched > 0 && (
                      <span> ({signatureStats.fetched}⬆)</span>
                    )}
                  </span>
                </td>
              </tr>
              <tr>
                <td>Transactions</td>
                <td className="responsive-td">
                  <span>
                    {txnStats.total}{" "}
                    {txnStats.fetched > 0 && (
                      <span> ({txnStats.fetched}⬆)</span>
                    )}
                  </span>
                </td>
              </tr>
            </tbody>
          </table>
        </CardBody>
      </Card>
      <div className="horizontal-line"></div>
    </>
  );
}
