Name:           gic
Version:        1.0.0
Release:        1%{?dist}
Summary:        High-performance DevOps TUI Editor & Intelligence Platform

License:        MIT or Apache-2.0
URL:            https://github.com/Aakashvish187/gic
Source0:        %{name}-%{version}.tar.gz


%description
GIC (Infrastructure-as-Code Editor) is a modern terminal-based editor
optimized for Kubernetes, Docker, Terraform, Ansible, and GitHub Actions
configurations with built-in schema validation, starter wizards, and IntelliSense.

%prep
%setup -q

%build
cargo build --release

%install
rm -rf $RPM_BUILD_ROOT
mkdir -p $RPM_BUILD_ROOT/%{_bindir}
cp target/release/gic $RPM_BUILD_ROOT/%{_bindir}/

%files
%{_bindir}/gic
%doc README.md
%license LICENSE

%changelog
* Wed Jul 29 2026 Aakash Vishwakarma <aakash@devops.org> - 1.0.0-1
- GIC v1.0.0 Production Release
