using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_PlacedBuildExposed
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.PlacedBuildExposed); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.PlacedBuildExposed)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize TotalBuildTime
            s.Write(value.TotalBuildTime);
            //  Serialize ElapsedBuildTime
            s.Write(value.ElapsedBuildTime);
            //  Serialize IsPaused
            s.Write(value.IsPaused);
            //  Serialize BuildSpeed
            s.Write(value.BuildSpeed);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.PlacedBuildExposed)) as Rts.CnC.Messages.Client.PlacedBuildExposed;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize TotalBuildTime
            s.Read(out value.TotalBuildTime);
            //  Deserialize ElapsedBuildTime
            s.Read(out value.ElapsedBuildTime);
            //  Deserialize IsPaused
            s.Read(out value.IsPaused);
            //  Deserialize BuildSpeed
            s.Read(out value.BuildSpeed);

            return value;
        }
        
    }
}
