import { Suspense } from "react";
import MapCreationPage from "./features/map-creation-tool/page";

function App() {
  return (
    <Suspense fallback={<p>Loading...</p>}>
      <MapCreationPage />
    </Suspense>
  );
}

export default App;
