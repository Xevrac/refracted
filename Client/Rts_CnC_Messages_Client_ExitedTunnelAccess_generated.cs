using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_ExitedTunnelAccess
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.ExitedTunnelAccess); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.ExitedTunnelAccess)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize TunnelAccessId
            s.Write(value.TunnelAccessId);
            //  Serialize ExitingEntityId
            s.Write(value.ExitingEntityId);
            //  Serialize Position
            s.Write(value.Position);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.ExitedTunnelAccess)) as Rts.CnC.Messages.Client.ExitedTunnelAccess;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize TunnelAccessId
            s.Read(out value.TunnelAccessId);
            //  Deserialize ExitingEntityId
            s.Read(out value.ExitingEntityId);
            //  Deserialize Position
            s.Read(out value.Position);

            return value;
        }
        
    }
}
