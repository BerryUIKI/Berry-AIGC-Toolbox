using BerryAIGen.Domain.Entities;
using Microsoft.EntityFrameworkCore;
using System.Reflection;

namespace BerryAIGen.Infrastructure.Data.Context;

/// <summary>
/// SQLite database context for the application.
/// </summary>
public class SqliteDbContext : DbContext
{
    /// <summary>
    /// Gets or sets the Images DbSet.
    /// </summary>
    public DbSet<Image> Images { get; set; } = null!;

    /// <summary>
    /// Gets or sets the Albums DbSet.
    /// </summary>
    public DbSet<Album> Albums { get; set; } = null!;

    /// <summary>
    /// Gets or sets the Tags DbSet.
    /// </summary>
    public DbSet<Tag> Tags { get; set; } = null!;

    /// <summary>
    /// Gets or sets the Folders DbSet.
    /// </summary>
    public DbSet<Folder> Folders { get; set; } = null!;

    /// <summary>
    /// Initializes a new instance of the <see cref="SqliteDbContext"/> class.
    /// </summary>
    public SqliteDbContext()
    {
    }

    /// <summary>
    /// Initializes a new instance of the <see cref="SqliteDbContext"/> class.
    /// </summary>
    /// <param name="options">The options for this context.</param>
    public SqliteDbContext(DbContextOptions<SqliteDbContext> options) : base(options)
    {
    }

    /// <summary>
    /// Configures the model that was discovered by convention from the entity types.
    /// </summary>
    /// <param name="modelBuilder">The builder being used to construct the model for this context.</param>
    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        base.OnModelCreating(modelBuilder);

        // Apply all configurations from the current assembly
        modelBuilder.ApplyConfigurationsFromAssembly(Assembly.GetExecutingAssembly());

        // Configure Image entity
        ConfigureImage(modelBuilder);

        // Configure Album entity
        ConfigureAlbum(modelBuilder);

        // Configure Tag entity
        ConfigureTag(modelBuilder);

        // Configure Folder entity
        ConfigureFolder(modelBuilder);
    }

    private static void ConfigureImage(ModelBuilder modelBuilder)
    {
        modelBuilder.Entity<Image>(entity =>
        {
            entity.ToTable("Images");

            // Configure primary key
            entity.HasKey(e => e.Id);

            // Configure properties
            entity.Property(e => e.Path)
                .IsRequired()
                .HasMaxLength(2048);

            entity.Property(e => e.Hash)
                .IsRequired()
                .HasMaxLength(128);

            entity.Property(e => e.ImageType)
                .IsRequired()
                .HasMaxLength(20);

            entity.Property(e => e.FileSize)
                .IsRequired();

            entity.Property(e => e.CreatedAt)
                .IsRequired();

            entity.Property(e => e.ModifiedAt)
                .IsRequired();

            // Configure value objects
            entity.OwnsOne(e => e.Dimensions, dim =>
            {
                dim.Property(d => d.Width).IsRequired();
                dim.Property(d => d.Height).IsRequired();
            });

            entity.OwnsOne(e => e.Rating, r =>
            {
                r.Property(r => r.Value).IsRequired();
            });

            // Configure relationships
            entity.HasOne(e => e.FolderId)
                .WithMany()
                .HasForeignKey("FolderId")
                .IsRequired();

            // Configure many-to-many relationships (via shadow properties)
            entity.HasMany("Tags")
                .WithMany()
                .UsingEntity("ImageTags",
                    j => j.HasOne(typeof(Tag)).WithMany().HasForeignKey("TagId"),
                    j => j.HasOne(typeof(Image)).WithMany().HasForeignKey("ImageId"),
                    j => j.HasKey("ImageId", "TagId"));

            entity.HasMany("Albums")
                .WithMany()
                .UsingEntity("AlbumImages",
                    j => j.HasOne(typeof(Album)).WithMany().HasForeignKey("AlbumId"),
                    j => j.HasOne(typeof(Image)).WithMany().HasForeignKey("ImageId"),
                    j =>
                    {
                        j.HasKey("ImageId", "AlbumId");
                        j.Property<int>("SortOrder").HasDefaultValue(0);
                    });

            // Create indexes
            entity.HasIndex(e => e.Path).IsUnique();
            entity.HasIndex(e => e.Hash);
            entity.HasIndex(e => e.CreatedAt);
            entity.HasIndex("FolderId");
        });
    }

    private static void ConfigureAlbum(ModelBuilder modelBuilder)
    {
        modelBuilder.Entity<Album>(entity =>
        {
            entity.ToTable("Albums");

            entity.HasKey(e => e.Id);

            entity.Property(e => e.Name)
                .IsRequired()
                .HasMaxLength(200);

            entity.Property(e => e.Description)
                .HasMaxLength(2000);

            entity.Property(e => e.CreatedAt).IsRequired();
            entity.Property(e => e.ModifiedAt).IsRequired();

            // Create indexes
            entity.HasIndex(e => e.Name).IsUnique();
        });
    }

    private static void ConfigureTag(ModelBuilder modelBuilder)
    {
        modelBuilder.Entity<Tag>(entity =>
        {
            entity.ToTable("Tags");

            entity.HasKey(e => e.Id);

            entity.Property(e => e.Name)
                .IsRequired()
                .HasMaxLength(100);

            entity.Property(e => e.Color)
                .HasMaxLength(10);

            entity.Property(e => e.Category)
                .HasMaxLength(50);

            entity.Property(e => e.CreatedAt).IsRequired();

            // Create indexes
            entity.HasIndex(e => e.Name).IsUnique();
        });
    }

    private static void ConfigureFolder(ModelBuilder modelBuilder)
    {
        modelBuilder.Entity<Folder>(entity =>
        {
            entity.ToTable("Folders");

            entity.HasKey(e => e.Id);

            entity.Property(e => e.Path)
                .IsRequired()
                .HasMaxLength(2048);

            entity.Property(e => e.CreatedAt).IsRequired();
            entity.Property(e => e.ModifiedAt).IsRequired();

            // Configure self-referencing relationship
            entity.HasOne<Folder>()
                .WithMany()
                .HasForeignKey(e => e.ParentFolderId)
                .OnDelete(DeleteBehavior.Restrict);

            // Create indexes
            entity.HasIndex(e => e.Path).IsUnique();
        });
    }
}