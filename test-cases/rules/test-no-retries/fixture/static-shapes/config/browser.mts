const settings = {
  workers: inCI ? 2 : 1,
  fullyParallel: inCI ? false : true,
  projects: [
    { name: projectName, retries: 2.5 },
    { name: "unlimited", retries: 1e30 },
  ],
};

export default settings;
