import { useEffect, useState } from "react";
import { Card, CardHeader, CardBody } from "./Card";

export default function IndexingUpdates({
  address,
  txnsIndexed,
  setAccount,
  settxnsIndexed,
  setError,
}) {
  let [accountFetched, setAccountFetched] = useState(false);
  let [loading, setLoading] = useState(true);
  let [err, setErr] = useState(false);

  useEffect(() => {
    const sse = new EventSource(
      `http://localhost:5000/api/accounts/${address}/index/sse`
    );

    sse.addEventListener("account-fetched", (e) => {
      setAccount(JSON.parse(e.data));
      setAccountFetched(true);
    });
    sse.addEventListener("transactions-fetched", (e) => {
      settxnsIndexed(JSON.parse(e.data));
    });
    sse.addEventListener("error", (e) => {
      sse.close();
      setErr(true);
      setError(e.data);
      setLoading(false);
    });
    sse.addEventListener("close", (e) => {
      console.log(e.data);
      sse.close();
      setLoading(false);
    });

    return () => {
      sse.close();
    };
  }, [address, setAccount, settxnsIndexed, setError]);

  if (err) {
    return <></>;
  }

  return (
    <Card>
      <CardHeader>Indexing Updates</CardHeader>
      <CardBody>
        <table>
          <tbody>
            <tr>
              <td>Indexing Status</td>
              <td style={{ textAlign: "center" }}>
                {loading ? "🟠 Running..." : "🟢 Completed"}
              </td>
            </tr>
            <tr>
              <td>Account Fetched</td>
              <td style={{ textAlign: "center" }}>
                {accountFetched ? "✅" : "❌"}
              </td>
            </tr>
            <tr>
              <td>Transactions Fetched</td>
              <td style={{ textAlign: "center" }}>{txnsIndexed}</td>
            </tr>
          </tbody>
        </table>
      </CardBody>
    </Card>
  );
}
