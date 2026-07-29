use crate::starter_engine::models::{GeneratedFile, ProjectConfig, TemplateGenerator};

pub struct AnsibleStarter;

impl TemplateGenerator for AnsibleStarter {
    fn generate(&self, config: &ProjectConfig) -> Vec<GeneratedFile> {
        let playbook = config
            .get_answer("playbook")
            .map(|s| s.as_str())
            .unwrap_or("Docker Install");

        let content = match playbook {
            "Docker Install" => {
                r#"---
- name: Install Docker
  hosts: all
  become: yes
  tasks:
    - name: Update apt cache
      apt:
        update_cache: yes
        cache_valid_time: 3600

    - name: Install dependencies
      apt:
        name:
          - apt-transport-https
          - ca-certificates
          - curl
          - software-properties-common
        state: present

    - name: Add Docker GPG key
      apt_key:
        url: https://download.docker.com/linux/ubuntu/gpg
        state: present

    - name: Add Docker APT repository
      apt_repository:
        repo: deb [arch=amd64] https://download.docker.com/linux/ubuntu focal stable
        state: present

    - name: Install Docker CE
      apt:
        name: docker-ce
        state: present

    - name: Ensure Docker is started and enabled
      systemd:
        name: docker
        state: started
        enabled: yes
"#
            }
            "Nginx Install" => {
                r#"---
- name: Install and Configure Nginx
  hosts: webservers
  become: yes
  tasks:
    - name: Install Nginx
      apt:
        name: nginx
        state: latest
        update_cache: yes
    
    - name: Start Nginx service
      service:
        name: nginx
        state: started
        enabled: yes
"#
            }
            "Create User" => {
                r#"---
- name: Create a new user
  hosts: all
  become: yes
  vars:
    username: "deploy"
  tasks:
    - name: Create user {{ username }}
      user:
        name: "{{ username }}"
        state: present
        shell: /bin/bash
        createhome: yes
"#
            }
            "SSH Hardening" => {
                r#"---
- name: Harden SSH
  hosts: all
  become: yes
  tasks:
    - name: Disable Root Login
      lineinfile:
        dest: /etc/ssh/sshd_config
        regexp: '^PermitRootLogin'
        line: 'PermitRootLogin no'
        state: present
      notify: Restart SSH

    - name: Disable Password Authentication
      lineinfile:
        dest: /etc/ssh/sshd_config
        regexp: '^PasswordAuthentication'
        line: 'PasswordAuthentication no'
        state: present
      notify: Restart SSH

  handlers:
    - name: Restart SSH
      service:
        name: sshd
        state: restarted
"#
            }
            "Deploy App" => {
                r#"---
- name: Deploy Application
  hosts: app_servers
  become: yes
  tasks:
    - name: Copy application code
      copy:
        src: ./app/
        dest: /var/www/app/
        owner: www-data
        group: www-data
    
    - name: Restart application service
      systemd:
        name: myapp
        state: restarted
"#
            }
            _ => {
                r#"---
- name: Generic Playbook
  hosts: all
  tasks:
    - name: Ping
      ping:
"#
            }
        };

        vec![GeneratedFile {
            path: "playbook.yml".to_string(),
            content: content.to_string(),
        }]
    }
}
