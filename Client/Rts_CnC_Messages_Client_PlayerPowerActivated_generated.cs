using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_PlayerPowerActivated
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.PlayerPowerActivated); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.PlayerPowerActivated)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize AbilityId
            s.Write(value.AbilityId);
            //  Serialize MillisecondsToReenable
            s.Write(value.MillisecondsToReenable);
            //  Serialize DelayBeforeActivation
            s.Write(value.DelayBeforeActivation);
            //  Serialize Flags
            s.Write(value.Flags);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.PlayerPowerActivated)) as Rts.CnC.Messages.Client.PlayerPowerActivated;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize AbilityId
            s.Read(out value.AbilityId);
            //  Deserialize MillisecondsToReenable
            s.Read(out value.MillisecondsToReenable);
            //  Deserialize DelayBeforeActivation
            s.Read(out value.DelayBeforeActivation);
            //  Deserialize Flags
            s.Read(out value.Flags);

            return value;
        }
        
    }
}
