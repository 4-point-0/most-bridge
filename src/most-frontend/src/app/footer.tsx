"use client";

const Footer = () => {
  return (
    <footer className="z-100 sticky bottom-0 mx-auto mt-2 h-12 w-[100%] max-w-2xl border-t-2 border-[border] px-4 py-8">
      <div className="flex h-full w-full items-center justify-center gap-14">
        <p className="leading-7 [&:not(:first-child)]:mt-6">
          Powered by @{" "}
          <a target="_blank" href="https://www.4pto.io/">
            4pto
          </a>
        </p>
      </div>
    </footer>
  );
};

export default Footer;
