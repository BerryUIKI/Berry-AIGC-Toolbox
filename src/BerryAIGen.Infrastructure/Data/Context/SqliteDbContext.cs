using BerryAIGen.Domain.Entities;
using BerryAIGen.Infrastructure.Data.Converters;
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

        // Ignore strongly-typed ID types so EF Core doesn't try to treat them as entities
        modelBuilder.Ignore<Domain.ValueObjects.ImageId>();
        modelBuilder.Ignore<Domain.ValueObjects.AlbumId>();
        modelBuilder.Ignore<Domain.ValueObjects.TagId>();
        modelBuilder.Ignore<Domain.ValueObjects.FolderId>();

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

            // Configure primary key with strongly-typed ID converter
            entity.HasKey(e => e.Id);
            entity.Property(e => e.Id)
                .HasConversion(new StronglyTypedIdConverter<Domain.ValueObjects.ImageId>());

            // Configure properties
            entity.Property(e => e.Path)
                .HasConversion<FilePathConverter>()
                .IsRequired()
                .HasMaxLength(2048);

            entity.Property(e => e.Hash)
                .HasConversion<HashConverter>()
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

            // Configure value objects as owned entities
            entity.OwnsOne(e => e.Dimensions, dim =>
            {
                dim.Property(d => d.Width).IsRequired();
                dim.Property(d => d.Height).IsRequired();
            });

            // Configure Rating as a simple property with converter (not owned entity)
            entity.Property(e => e.Rating)
                .HasConversion<RatingConverter>()
                .IsRequired();

            // Configure FolderId as a simple property (stored as GUID)
            entity.Property(e => e.FolderId)
                .HasConversion(new StronglyTypedIdConverter<Domain.ValueObjects.FolderId>())
                .IsRequired();

            // Create indexes
            entity.HasIndex(e => e.Path).IsUnique();
            entity.HasIndex(e => e.Hash);
            entity.HasIndex(e => e.CreatedAt);
            entity.HasIndex(e => e.FolderId);
        });
    }

    private static void ConfigureAlbum(ModelBuilder modelBuilder)
    {
        modelBuilder.Entity<Album>(entity =>
        {
            entity.ToTable("Albums");

            entity.HasKey(e => e.Id);
            entity.Property(e => e.Id)
                .HasConversion(new StronglyTypedIdConverter<Domain.ValueObjects.AlbumId>());

            entity.Property(e => e.Name)
                .IsRequired()
                .HasMaxLength(200);

            entity.Property(e => e.Description)
                .HasMaxLength(2000);

            entity.Property(e => e.CreatedAt).IsRequired();
            entity.Property(e => e.ModifiedAt).IsRequired();

            // Configure CoverImageId as optional
            entity.Property(e => e.CoverImageId)
                .HasConversion(new NullableStronglyTypedIdConverter<Domain.ValueObjects.ImageId>());

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
            entity.Property(e => e.Id)
                .HasConversion(new StronglyTypedIdConverter<Domain.ValueObjects.TagId>());

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
            entity.Property(e => e.Id)
                .HasConversion(new StronglyTypedIdConverter<Domain.ValueObjects.FolderId>());

            entity.Property(e => e.Path)
                .HasConversion<FilePathConverter>()
                .IsRequired()
                .HasMaxLength(2048);

            // Configure ParentFolderId as optional
            entity.Property(e => e.ParentFolderId)
                .HasConversion(new NullableStronglyTypedIdConverter<Domain.ValueObjects.FolderId>());

            entity.Property(e => e.CreatedAt).IsRequired();
            entity.Property(e => e.ModifiedAt).IsRequired();

            // Create indexes
            entity.HasIndex(e => e.Path).IsUnique();
        });
    }
}