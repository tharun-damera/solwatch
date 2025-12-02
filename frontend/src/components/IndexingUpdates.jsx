import { useEffect, useState } from "react";
import { Card, CardHeader, CardBody } from "./Card";

export default function IndexingUpdates({
  address,
  signaturesIndexed,
  txnsIndexed,
  setAccount,
  setSignaturesIndexed,
  setTxnsIndexed,
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
    sse.addEventListener("signatures-fetched", (e) => {
      setSignaturesIndexed(JSON.parse(e.data));
    });
    sse.addEventListener("transactions-fetched", (e) => {
      setTxnsIndexed(JSON.parse(e.data));
    });
    sse.addEventListener("error", (e) => {
      sse.close();
      setErr(true);
      setError(e.data);
      setLoading(false);
    });
    sse.addEventListener("close", () => {
      sse.close();
      setLoading(false);
    });

    return () => {
      sse.close();
    };
  }, [address, setAccount, setSignaturesIndexed, setTxnsIndexed, setError]);

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
              <td>Signatures Fetched</td>
              <td style={{ textAlign: "center" }}>{signaturesIndexed}</td>
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
